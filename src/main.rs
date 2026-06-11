fn main() {
    microps_rs::ProtocolStackApp::new()
        .unwrap()
        .setup_mock()
        .unwrap()
        .run()
        .unwrap();
}
