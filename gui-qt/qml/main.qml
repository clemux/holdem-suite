import QtQuick.Controls 2.12
import QtQuick.Controls.Material 2.12
import QtQuick.Layouts
import QtQuick.Window 2.12

import demo 1.0

Window {
    title: qsTr("Hello App")
    visible: true
    height: 480
    width: 640

    Summary {
        id: summary1
        tournamentId: 42
        name: "Summary 1"
    }

    Summaries {
        id: summaries
    }


    TableView {
        anchors.fill: parent
        id: summaryView
        columnSpacing: 1
        rowSpacing: 1
        clip: true
        model: TableModel {
        }


        delegate: Rectangle {
            implicitWidth: 100
            implicitHeight: 50
            border.width: 1

            Text {
                text: display
            }
        }
    }

    Button {
        Layout.row: 1
        Layout.column: 1
        text: "Click me!"

        onClicked: {
            this.text = summary1.name
            summaryView.model.loadSummaries()
        }
    }
}