// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "HermesApp",
    platforms: [
        .iOS(.v17)
    ],
    targets: [
        .target(
            name: "HermesApp",
            path: "HermesApp",
            resources: [
                .process("Assets")
            ]
        ),
        .testTarget(
            name: "HermesAppTests",
            dependencies: [
                "HermesApp",
            ],
            path: "HermesAppTests"
        ),
    ]
)
