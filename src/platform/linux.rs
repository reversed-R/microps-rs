pub(crate) mod driver;
pub(crate) mod page;
pub(crate) mod time;

use std::sync::Arc;

use crate::devices::NetDevice;

static IRQS: Irqs = Irqs::new();

const INTR_IRQ_SHARED: u16 = 0x0001;

#[derive(Debug)]
struct IrqEntry {
    irq: u32,
    flags: u16,
    dev: Arc<dyn NetDevice>,
}

struct Irqs {
    irqs: Vec<IrqEntry>,
}

impl Irqs {
    const fn new() -> Self {
        Self { irqs: Vec::new() }
    }
}
