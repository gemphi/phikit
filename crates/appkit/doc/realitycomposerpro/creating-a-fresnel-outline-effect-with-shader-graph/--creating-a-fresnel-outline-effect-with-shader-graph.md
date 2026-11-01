> [REALITYCOMPOSERPRO](../../realitycomposerpro/-realitycomposerpro.md) > [CREATING A FRESNEL OUTLINE EFFECT WITH SHADER GRAPH](../../realitycomposerpro/creating-a-fresnel-outline-effect-with-shader-graph/--creating-a-fresnel-outline-effect-with-shader-graph.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../realitykit/-realitykit.md)
- [🛠 Reality Composer Pro](../../realitycomposerpro/-realitycomposerpro.md)
- [🕶 visionOS Home](../../visionos/-visionos.md)

---

# Creating a Fresnel outline effect with Shader Graph

Highlight a model by adding an outline effect.

## Overview

Emphasizing the edges of a model with an outline is an effective way to highlight and call attention to it, especially when it’s the target of a person’s gaze or interaction.

One way to add an outline to a model is to approximate the Fresnel effect, wherein the angle between the view direction and the surface normal of a model affects the shading around its edges. Specifically, you can highlight the edges of a model by taking the dot product between the view direction and the surface normal direction and applying an emissive color to the areas where the dot product is close to zero (that is, the vectors are perpendicular). See [Loading entities with ShaderGraph materials](../../realitycomposerpro/loading-entities-with-shadergraph-materials/--loading-entities-with-shadergraph-materials.md) to download a sample project containing this shader along with many others.

## Prepare the Fresnel node graph

Set up a reusable [NodeGraph](../../https_//developer.apple.com/documentation/shadergraph/material/nodegraph/-------nodegraph.md) to house your Fresnel logic by following these steps:

The following image shows the main body of the Shader Graph material:

![A screenshot of the Shader Graph described above. On the left is a NodeGraph node with the name Fresnel, followed by a Preview Surface node whose Out output connects to the Custom Surface input of the Outputs node on the right.](./../../assets/fresnel-1.png)

## Compose the Fresnel logic

Approximate the Fresnel effect by taking the dot product between the view direction and the normal direction in the Fresnel node graph:

The interior of the Fresnel node graph is as follows:

![A screenshot of the Shader Graph described above. The output of a View Direction node connects to the top input of a Dot Product node. The output of a Normal node connects to the bottom input of the same Dot Product node. The output of the Dot Product node connects to the In input of a Clamp node, whose output connects to a One Minus node, which in turn connects to the top input of a Power node. A uniform input Float node with the name Falloff connects to the bottom input of the same power node. The output of the Power node connects to the Out input of Outputs.](./../../assets/fresnel-2.png)

The following images show the result of each stage of the Fresnel node graph if it were fully connected:

In this case, the view direction and the normal direction are parallel at the center of the sphere, and become more perpendicular to each other toward the edges of the sphere, meaning the dot product ranges from a value of 1 at the center to a value of 0 near the edges. You can flip this by subtracting 1 from the dot product, and increase the rate of falloff by taking that value to a power.

## Create an outline effect with the Fresnel node

One way to create an outline effect with the  node is by employing its output to apply an emissive color around the edges of a model. In the main graph of your shader:

The resulting graph is as follows:

![A screenshot of the Shader Graph described above. The output of a uniform input Color3 (Float) node with the name OutlineColor connects to the top input of a Multiply node. The Out output of the Fresnel node graph node connects to the bottom input of the same Multiply node. The output of the Multiply node connects to the Emissive Color input of a Preview Surface node, whose Out output connects to the Custom Surface input of the Outputs node.](./../../assets/fresnel-6.png)

You can adjust the  input to change the color of the outline, and you can adjust the  to control the rate of falloff for the Fresnel effect, as shown in the three examples below:

