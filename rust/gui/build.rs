use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("OpenBlade.Core"))
        .qt_module("Gui")
        .qt_module("Quick")
        .files(["src/core.rs", "src/main.rs"])
        .cpp_file("cpp/ob_helper.cpp")
        .include_dir("cpp")
        .build();
}
