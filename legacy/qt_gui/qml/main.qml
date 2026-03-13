import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: root
    visible: true
    width: 800
    height: 600
    title: "Listory Search"

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 10

        // Search bar
        RowLayout {
            Layout.fillWidth: true
            spacing: 10

            TextField {
                id: searchInput
                Layout.fillWidth: true
                placeholderText: "🔍 输入搜索关键词..."
                font.pixelSize: 14

                Keys.onReturnPressed: {
                    searchBackend.search(searchInput.text)
                }
            }

            Button {
                text: "查找"
                font.pixelSize: 14
                onClicked: {
                    searchBackend.search(searchInput.text)
                }
            }
        }

        // Results table
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            border.color: "#cccccc"
            border.width: 1

            ListView {
                id: resultsView
                anchors.fill: parent
                anchors.margins: 1
                clip: true

                model: searchBackend

                header: Rectangle {
                    width: resultsView.width
                    height: 30
                    color: "#f0f0f0"

                    Row {
                        anchors.fill: parent

                        Text {
                            width: parent.width * 0.3
                            height: parent.height
                            text: "文件名"
                            font.bold: true
                            font.pixelSize: 13
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: 10
                        }

                        Rectangle {
                            width: 1
                            height: parent.height
                            color: "#cccccc"
                        }

                        Text {
                            width: parent.width * 0.7 - 1
                            height: parent.height
                            text: "完整路径"
                            font.bold: true
                            font.pixelSize: 13
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: 10
                        }
                    }
                }

                delegate: Rectangle {
                    width: resultsView.width
                    height: 35
                    color: index % 2 === 0 ? "#ffffff" : "#f9f9f9"

                    MouseArea {
                        anchors.fill: parent
                        hoverEnabled: true

                        onEntered: parent.color = "#e3f2fd"
                        onExited: parent.color = index % 2 === 0 ? "#ffffff" : "#f9f9f9"

                        onDoubleClicked: {
                            searchBackend.openFile(index)
                        }
                    }

                    Row {
                        anchors.fill: parent

                        Text {
                            width: parent.width * 0.3
                            height: parent.height
                            text: model.filename
                            font.pixelSize: 12
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: 10
                            elide: Text.ElideRight
                        }

                        Rectangle {
                            width: 1
                            height: parent.height
                            color: "#eeeeee"
                        }

                        Text {
                            width: parent.width * 0.7 - 1
                            height: parent.height
                            text: model.path
                            font.pixelSize: 12
                            color: "#666666"
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: 10
                            elide: Text.ElideMiddle
                        }
                    }
                }

                ScrollBar.vertical: ScrollBar {
                    policy: ScrollBar.AsNeeded
                }
            }
        }

        // Status bar
        Rectangle {
            Layout.fillWidth: true
            height: 30
            color: "#f5f5f5"
            border.color: "#cccccc"
            border.width: 1

            Row {
                anchors.fill: parent
                anchors.leftMargin: 10
                anchors.rightMargin: 10
                spacing: 20

                Text {
                    height: parent.height
                    text: "Engine: " + searchBackend.engineStatus
                    font.pixelSize: 12
                    verticalAlignment: Text.AlignVCenter
                }

                Rectangle {
                    width: 1
                    height: parent.height * 0.6
                    anchors.verticalCenter: parent.verticalCenter
                    color: "#cccccc"
                }

                Text {
                    height: parent.height
                    text: "Files: " + searchBackend.fileCount
                    font.pixelSize: 12
                    verticalAlignment: Text.AlignVCenter
                }
            }
        }
    }
}
