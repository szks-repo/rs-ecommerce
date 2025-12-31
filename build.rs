fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("protoc path");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    let mut config = prost_build::Config::new();
    config.compile_well_known_types();
    config.extern_path(".google.protobuf", "::pbjson_types");

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let descriptor_path = out_dir.join("descriptor.bin");
    config.file_descriptor_set_path(&descriptor_path);

    let protos = &[
        "proto/ecommerce/v1/common.proto",
        "proto/ecommerce/v1/storefront.proto",
        "proto/ecommerce/v1/backoffice.proto",
        "proto/ecommerce/v1/store_settings.proto",
        "proto/ecommerce/v1/setup.proto",
        "proto/ecommerce/v1/audit.proto",
        "proto/ecommerce/v1/auth.proto",
        "proto/ecommerce/v1/store_staff.proto",
        "proto/ecommerce/v1/permissions.proto",
        "proto/ecommerce/v1/identity.proto",
        "proto/ecommerce/v1/customer.proto",
        "proto/ecommerce/v1/auction.proto",
    ];
    let includes = &["proto"];

    config
        .compile_protos(protos, includes)
        .expect("compile protos");

    pbjson_build::Builder::new()
        .register_descriptors(&std::fs::read(descriptor_path).expect("read descriptor.bin"))
        .expect("register descriptors")
        .build(&["."])
        .expect("build pbjson");
}
