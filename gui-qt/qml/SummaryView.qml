import QtQuick
import QtQuick.Controls 2.12
import Qt.labs.qmlmodels 1.0

TableView {
    columnSpacing: 1
    rowSpacing: 1
    clip: true


    delegate: Rectangle {
        implicitWidth: 100
        implicitHeight: 50
        border.width: 1

        Text {
            text: display
            anchors.centerIn: parent
        }
    }
}