> [REALITYCOMPOSERPRO](../../realitycomposerpro/-realitycomposerpro.md) > [BUILD AND RUN REALITY COMPOSER PRO SCENES IN YOUR APP](../../realitycomposerpro/build-and-run-reality-composer-pro-scenes-in-your-app/--build-and-run-reality-composer-pro-scenes-in-your-app.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../realitykit/-realitykit.md)
- [🛠 Reality Composer Pro](../../realitycomposerpro/-realitycomposerpro.md)
- [🕶 visionOS Home](../../visionos/-visionos.md)

---

# Building and running Reality Composer Pro scenes in your app

Preview scenes on your visionOS device and learn how to load them in your app.

## Overview

During development, your Reality Composer Pro projects change over time and may get more complex. You can preview your scene directly in Reality Composer Pro on a visionOS device without having to build and run in Xcode. If you’re ready to test how your content runs in your app, you can build and run your app with either Xcode’s visionOS simulator or on a physical device.

### Preview scenes on a device

If you have an Apple Vision Pro connected to your Mac, you can preview your scene on the device by selecting Preview > Play on the menu bar or by clicking on the Preview button in the Reality Composer Pro toolbar, represented by an Apple Vision Pro icon. If you have multiple Apple Vision Pro devices connected, choose which device to use by clicking the menu next to the Preview button.

### Load scenes in your app’s view

You can load a Reality Composer Pro scene similar to how you load a  asset from your app bundle. The difference is you have to specify the Reality Composer Pro package bundle instead. You can do this in the  closure of a [RealityView](../../realitykit/realityview/--realityview.md) initializer.

Each Reality Composer Pro package defines a global constant that points to its bundle. The system creates the bundle name by appending “Bundle” to the end of the project’s name.

In the default visionOS template in Xcode, the Reality Composer Pro project is called , with the global bundle variable called . The following code shows how to load the default bundle into a :

```swift
    RealityView { content in
        if let scene = try? await Entity.load(named: "Biplane", 
            in: realityKitContentBundle) {
                myDataModel.add(scene) 
                content.add(scene)
                } update: { content in
                    // ...
                    }
    }
```

When RealityKit finishes loading the scene, the  variable contains the root entity of the specified scene and includes this content in , which is then displayed by  to the user.

### Run the App with the Reality Composer Pro project

To build and run your Xcode project, choose Product > Run, or click the Run button in your project’s toolbar. You can test your app’s performance with either the visionOS simulator or a connected physical device.

