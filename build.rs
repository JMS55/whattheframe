use gtk4::gio::compile_resources;

fn main() {
    compile_resources("res", "res/resources.gresource.xml", "compiled.gresource");
}
