extern crate embed_resource;

fn main() {
    if std::env::var("TARGET").unwrap().contains("windows") {
        embed_resource::compile("icon.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}
