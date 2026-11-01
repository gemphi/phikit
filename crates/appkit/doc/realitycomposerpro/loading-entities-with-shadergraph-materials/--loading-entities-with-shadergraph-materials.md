> [REALITYCOMPOSERPRO](../../realitycomposerpro/-realitycomposerpro.md) > [LOADING ENTITIES WITH SHADERGRAPH MATERIALS](../../realitycomposerpro/loading-entities-with-shadergraph-materials/--loading-entities-with-shadergraph-materials.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../realitykit/-realitykit.md)
- [🛠 Reality Composer Pro](../../realitycomposerpro/-realitycomposerpro.md)
- [🕶 visionOS Home](../../visionos/-visionos.md)

---

# Loading entities with ShaderGraph materials

Bring entities that contain materials created with Reality Composer Pro for use in your visionOS app.

## Overview

Reality Composer Pro gives you the ability to create entities that you can bring over to, and interact with, in your visionOS app. This sample loads the entities created with [ShaderGraph](../../https_//developer.apple.com/documentation/shadergraph/-----shadergraph.md) and places them in a [ScrollView](../../https_//developer.apple.com/documentation/swiftui/scrollview/------scrollview.md).

![A screenshot showing three objects containing a shader graph material on them in a scroll view. The first one from the left is a blue object with vertex displacement with normal correction. The second from the left is a gradient shader. The third one from the left is a lava shader.](./../../assets/creating-materials-overview-image.jpg)

## Create the materials in ShaderGraph

In the Project Navigator, open Packages > RealityKitContent, select , then click Open in Reality Composer Pro.

![A screenshot.](./../../assets/open-package-in-reality-composer-pro.png)

The  package in the sample includes these shaders:

## Create a volumetric view in the app

The sample uses a volumetric view in a [WindowGroup](../../https_//developer.apple.com/documentation/swiftui/windowgroup/------windowgroup.md) to load the entities from the  package:

```swift
struct ShaderSamplesApp: App {
    var body: some Scene {
        WindowGroup {
            ShaderSamplesView()
        }
        .windowStyle(.volumetric)
    }
}
```

## Load the shaders from the content package

The  method loads all the shader entities containing the shader materials from the  scene in the :

```swift
private func loadShaders() async {
    guard
        let shaderSamplesSceneRoot = try? await Entity(
            named: "ShaderSamplesScene", in: realityKitContentBundle
        ).children.first
    else {
        return
    }

    // Get the shader sample entities.
    for shaderSampleEntity in shaderSamplesSceneRoot.children {
        shaderSampleEntities.append(shaderSampleEntity)
    }
}
```

The method saves the entities in a list named  to be used later to create the [RealityViewContent](../../https_//developer.apple.com/documentation/realitykit/realityviewcontent/------realityviewcontent.md).

## Add the shader entities into the RealityView content

The   method in the sample creates a [RealityView](../../https_//developer.apple.com/documentation/realitykit/realityview/------realityview.md) for an entity and adds it to the [RealityViewContent](../../https_//developer.apple.com/documentation/realitykit/realityviewcontent/------realityviewcontent.md) in the  method:

```swift
private func createRealityView(_ shaderSampleEntity: Entity) -> some View {
    RealityView { content in
        shaderSampleEntity.position = [0, 0, 0]
        shaderSampleEntity.scale = SIMD3<Float>(
            repeating: 0.75)
        content.add(shaderSampleEntity)
    }
    .glassBackgroundEffect()
    .containerRelativeFrame(
        [.horizontal], count: 3, spacing: 5)
}
```

The method positions the  at the center of the  and reduces the scale to fit in the [ScrollView](../../https_//developer.apple.com/documentation/swiftui/scrollview/------scrollview.md).

## Load the entities into a ScrollView

The sample uses a [ScrollView](../../https_//developer.apple.com/documentation/swiftui/scrollview/------scrollview.md) to display all the entities in the  list in a [RealityView](../../https_//developer.apple.com/documentation/realitykit/realityview/------realityview.md). The  shows the name of the shader below the :

```swift
var body: some View {
    ScrollView(.horizontal) {
        HStack {
            ForEach(shaderSampleEntities) { shaderSampleEntity in
                VStack {
                    // Display the shader sample entity.
                    createRealityView(shaderSampleEntity)

                    // Display the shader name.
                    createDisplayName(shaderSampleEntity)
                }.padding([.bottom])
            }
        }
    }.task {
        // Load the shader samples scene root.
        await loadShaders()
    }
}
```

The sample loads the shaders in an asynchronous [Task](../../https_//developer.apple.com/documentation/swift/task/------task.md), and displays the shaders in .

[Video: A video showing the shaders in a visionOS app in an immersive space. The immersive space is a furnished room, looking toward the back wall in the distance. The left wall has art hung on the wall, and there's a book case and window along the right wall. At the start of the video, the ScrollView contains three shaders — Outline, Fresnel, and Dissolve — and scrolls from right to left to reveal the Vertex Displacement, Normal Correction, Gradient, Lava, Toon, and PBR Toon shaders.](./../../assets/scrolling-through-shaders.mp4)

