#[cxx_qt::bridge]
mod my_object {
    use cxx_qt_lib::QString;
    use holdem_suite_parser::get_summaries;

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!(< QAbstractTableModel >);

        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
    }

    #[derive(Default)]
    #[cxx_qt::qobject(qml_uri = "demo", qml_version = "1.0")]
    pub struct Summary {
        #[qproperty]
        pub tournament_id: i32,
        #[qproperty]
        pub name: QString,
        #[qproperty]
        pub finish_place: i32,
    }

    impl qobject::Summary {
        #[qinvokable]
        pub fn say_hello(&self) {
            println!("Hello world!")
        }
    }

    #[derive(Default)]
    #[cxx_qt::qobject(qml_uri = "demo", qml_version = "1.0")]
    pub struct Summaries {}

    impl qobject::Summaries {
        #[qinvokable]
        pub fn say_hello(&self) {
            let results = get_summaries();
            for r in results {
                println!("{:?}", r)
            }
        }
    }

    #[cxx_qt::qobject(
        base = "QAbstractTableModel",
        qml_uri = "com.kdab.cxx_qt.demo",
        qml_version = "1.0"
    )]
    #[derive(Default)]
    pub struct TableModel {
        id: u32,
        v: Vec<(i32, i32, i32)>,
    }

    impl qobject::TableModel {
        #[qinvokable(cxx_override)]
        pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
            5
        }
        #[qinvokable(cxx_override)]
        pub fn column_count(&self, _parent: &QModelIndex) -> i32 {
            3
        }

        #[qinvokable(cxx_override)]
        fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
            println!("data: {:?}", self.rust().v.get(index.row() as usize));
            QVariant::from(&QString::from("1"))
        }

        #[qinvokable]
        pub fn load_summaries(&self) {
            let results = get_summaries();
            for r in results {
                println!("{:?}", r)
            }
        }
    }
}
