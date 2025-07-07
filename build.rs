fn main() {
    tonic_build::compile_protos("proto/order.proto").unwrap();
}
