use std::sync::{
    Arc, Mutex,
    atomic::{AtomicPtr, AtomicUsize, Ordering},
};

#[cfg(test)]
mod tests;

/// RCU (Read-Copy-Update) スタイルの共有コンテナ
///
/// [`RcuCell<T>`] は、
/// - 読み取り (`load()`) はロックなし
/// - 書き込み (`store()`, `update()`) はコピー差し替え, writer間Mutex, grace period待ちを行う
pub struct RcuCell<T> {
    /// Arc<T> の生ポインタ(stored ref = 強参照 1 個所有)
    ptr: AtomicPtr<T>,

    /// クリティカルセクション(Reader-1 ~ Reader-4)中の reader 数
    active_readers: AtomicUsize,

    /// writer 間の相互排他、直列化用
    /// readerは取得しない
    write_lock: Mutex<()>,
}

// SAFETY:
//   - AtomicPtr によりポインタへのアクセスはアトミック
//   - write_lock (Mutex) により writer は直列化される
//   - T: Send + Sync なら Arc<T> を複数スレッドで共有しても安全
unsafe impl<T: Send + Sync> Send for RcuCell<T> {}
unsafe impl<T: Send + Sync> Sync for RcuCell<T> {}

impl<T> RcuCell<T> {
    pub fn new(value: T) -> Self {
        let arc = Arc::new(value);
        // into_raw で Arc の stored ref を raw pointer に変換（strong_count はそのまま）
        let ptr = Arc::into_raw(arc) as *mut T;

        Self {
            ptr: AtomicPtr::new(ptr),
            active_readers: AtomicUsize::new(0),
            write_lock: Mutex::new(()),
        }
    }

    /// 現在の値のスナップショットをロックフリーで返す
    ///
    /// 返された `Arc<T>` は caller が保持する間、値の解放を阻止する
    /// writer が新しい値に差し替えた後も、この Arc が drop されるまで旧値は生き続ける
    pub fn load(&self) -> Arc<T> {
        // Reader-1:
        //   クリティカルセクション開始
        //   SeqCst が必要: T(1) < T(2) < T(A) を全順序で保証するため
        self.active_readers.fetch_add(1, Ordering::SeqCst);

        // Reader-2:
        //   ポインタ読み取り
        //   SeqCst が必要: old_ptr を観測した場合に T(2) < T(A) が成立するため
        let ptr = self.ptr.load(Ordering::SeqCst);

        // Reader-3:
        //   強参照を確保
        //
        //   SAFETY:
        //     - Reader-1 により active_readers >= 1
        //     - writer は grace period 1 で active_readers == 0 まで spin するため、
        //       old_arc を drop できない（strong_count >= 1 は保証済み）
        unsafe { Arc::increment_strong_count(ptr) };

        // Reader-4:
        //   クリティカルセクション終了
        //   Release で十分: Arc 操作との happens-before を確立すれば良い
        self.active_readers.fetch_sub(1, Ordering::Release);

        // Reader-5:
        //   所有済み Arc を返す
        //
        //   SAFETY:
        //     ptr は Reader-3 で increment した Arc と対応する
        unsafe { Arc::from_raw(ptr) }
    }

    /// 値を `value` で置き換える
    ///
    /// writer 間は直列化される。グレース期間(既存の `load()` 呼び出しが全て完了)
    /// まで旧値の解放は行われない
    pub fn store(&self, value: T) {
        let _guard = self.write_lock.lock().unwrap();
        self.store_locked(value);
    }

    /// 現在の値を参照して新しい値を生成し、原子的に差し替える
    ///
    /// `f` は旧値への参照を受け取り、新しい値（`T`）を返す
    ///
    /// # 注意
    ///
    /// `f` の中で `RcuCell::store()` や `RcuCell::update()` を呼ぶと
    /// `write_lock` でデッドロックする
    pub fn update<F: FnOnce(&T) -> T>(&self, f: F) {
        let _guard = self.write_lock.lock().unwrap();
        let snapshot = self.load();
        let new_val = f(&snapshot);
        // grace period の前に自分の snapshot を drop しないと、
        // store_locked の grace period 2（strong_count == 1 待ち）が
        // 永遠にブロックされる
        drop(snapshot);
        self.store_locked(new_val);
    }

    // write_lock 保持済みを前提
    //
    // grace period は クリティカルセクション中のreader だけを待つ
    // Arc スナップショットを保持している reader は待たない
    // Arc のリファレンスカウントが、最後の保持者が drop するまで旧値を生かし続ける
    fn store_locked(&self, value: T) {
        let new_arc = Arc::new(value);
        let new_ptr = Arc::into_raw(new_arc) as *mut T;

        // Writer-1:
        //   新値を公開
        //   SeqCst: reader の SeqCst load と全順序で整合させるため
        let old_ptr = self.ptr.swap(new_ptr, Ordering::SeqCst);

        // Writer-2:
        //   旧 Arc の stored ref を取得(所有権は我々が持つ)
        //
        //   SAFETY:
        //     old_ptr は直前まで AtomicPtr が保持していた stored ref
        //     from_raw は strong_count を変えず、所有権を Arc に戻すだけ
        let old_arc = unsafe { Arc::from_raw(old_ptr as *const T) };

        // Writer-3:
        //   grace period:
        //     クリティカルセクション中の reader(Reader-1 ~ Reader-4 区間)を待つ
        //     これにより、old_ptr の increment_strong_count を呼ぼうとしている reader が
        //     全員 Reader-3 を完了してから stored ref を解放できる
        //
        //     SeqCst が必要: reader の fetch_add(SeqCst) と全順序で整合させるため
        while self.active_readers.load(Ordering::SeqCst) > 0 {
            core::hint::spin_loop();
        }

        // stored ref を解放
        // スナップショットを保持している reader がいれば
        // Arc の refcount > 0 のまま残り、最後の reader が drop したときに解放される
        drop(old_arc);
    }
}

impl<T> Drop for RcuCell<T> {
    fn drop(&mut self) {
        let ptr = *self.ptr.get_mut();

        // SAFETY:
        //   ptr は new() で設定された stored ref
        //   RcuCell 破棄時に所有権を Arc に戻して解放する(リーク防止)
        //   もし reader が Arc を保持中であれば strong_count > 0 のまま残り、
        //   reader が drop するまでメモリは解放されない(Arc の通常の動作)
        let _ = unsafe { Arc::from_raw(ptr as *const T) };
    }
}
