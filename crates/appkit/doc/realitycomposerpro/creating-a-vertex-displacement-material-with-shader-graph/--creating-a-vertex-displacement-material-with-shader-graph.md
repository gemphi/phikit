> [REALITYCOMPOSERPRO](../../realitycomposerpro/-realitycomposerpro.md) > [CREATING A VERTEX DISPLACEMENT MATERIAL WITH SHADER GRAPH](../../realitycomposerpro/creating-a-vertex-displacement-material-with-shader-graph/--creating-a-vertex-displacement-material-with-shader-graph.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../realitykit/-realitykit.md)
- [🛠 Reality Composer Pro](../../realitycomposerpro/-realitycomposerpro.md)
- [🕶 visionOS Home](../../visionos/-visionos.md)

---

# Creating a vertex displacement material with Shader Graph

Animate the vertices of a mesh over time with 3D Perlin noise by creating a Shader Graph material.

## Overview

Moving the vertices of a mesh with a shader is a valuable technique to have in your shader toolbox. Using this technique, you can procedurally distort the shape of a mesh and animate it over time, all without having to author explicit frame-by-frame animations. Moving the vertices displaces them from their original positions, so this effect is known as , as shown in the example below:

[Video: A video of a toy biplane model whose vertices move and distort in shape over time.](./../../assets/vertex-displacement-0.mp4.mp4)

To create a shader graph material that displaces the vertices of a mesh over time, apply a positional offset to each vertex with a [Geometry Modifier](../../https_//developer.apple.com/documentation/shadergraph/realitykit/geometry-modifier-(realitykit)/-------geometry-modifier-(realitykit).md) node and a [Noise 3D](../../https_//developer.apple.com/documentation/shadergraph/3d-procedural/noise-3d/-------noise-3d.md) node. You can control the amplitude, scale, and rate of displacement with shader uniforms. See [Loading entities with ShaderGraph materials](../../realitycomposerpro/loading-entities-with-shadergraph-materials/--loading-entities-with-shadergraph-materials.md) to download a sample project containing this shader along with many others.

## Prepare a displacement node graph

House the vertex displacement logic in a reusable [NodeGraph](../../https_//developer.apple.com/documentation/shadergraph/material/nodegraph/-------nodegraph.md) by following these steps:

The following images show the Displacement node graph and its Inspector:

## Set up the geometry modifier

Add a [Geometry Modifier](../../https_//developer.apple.com/documentation/shadergraph/realitykit/geometry-modifier-(realitykit)/-------geometry-modifier-(realitykit).md) node to offset the vertex positions by the output of the Displacement node graph:

The main shader body is as follows:

![A Position node connects to the Position input of the Displacement node graph node, whose Model Position Offset output connects to the Model Position Offset input of a Geometry Modifier node. The output of the Geometry Modifier node connects to the Custom Geometry Modifier input of the Outputs node. The output of a Preview Surface node connects to the Custom Surface input of the Outputs node.](./../../assets/vertex-displacement-3.png)

In this example, the  of the  node is set to  so that the displacement effect remains consistent regardless of the model’s position, orientation, or scale in world space, but you can use  instead if you want your displacement effect to depend on these factors.

## Displace the vertices

One way to displace the vertices in a mesh is by utilizing output of a [Noise 3D](../../https_//developer.apple.com/documentation/shadergraph/3d-procedural/noise-3d/-------noise-3d.md) node:

The following image shows the interior of the Displacement node graph:

![A screenshot of the interior of the Displacement node graph described above. A uniform input Vector3 (Float) node connects to the Position input of a Noise 3D node, whose output connects to the Model Position Offset input of the Outputs node.](./../../assets/vertex-displacement-4.png)

You can see the result of the shader in the Shader Graph Preview window, as shown below:

![A screenshot of the Shader Graph preview window rendering a cube mesh. This technique displaces the mesh's vertices by noise so its shape appears distorted.](./../../assets/vertex-displacement-4-5.png)

In the example shown here, the  node generates vertex offsets with Perlin noise, which distorts the shape of the mesh in a smooth yet random manner. To distort the shape of the surface differently, you can experiment with other noise functions, such as [Fractal Noise 3D](../../https_//developer.apple.com/documentation/shadergraph/3d-procedural/fractal-noise-3d/-------fractal-noise-3d.md) or [Worley Noise 3D](../../https_//developer.apple.com/documentation/shadergraph/3d-procedural/worley-noise-3d/-------worley-noise-3d.md), or you can displace the vertices with your own custom logic.

## Control the amplitude of displacement

Control the degree to which the shader offsets the vertices by adjusting the  parameter of the  node:

The following image shows the interior of the Displacement node graph:

![A screenshot of the interior of the Displacement node graph described above. A uniform input Float node with the name Amplitude connects to a Convert node, which in turn connects to the Amplitude input of a Noise 3D node. A uniform input Vector3 (Float) node connects to tht Position input of the same Noise 3D node. The output of the Noise 3D node connects to the Model Position Offset input of the Outputs node.](./../../assets/vertex-displacement-5.png)

You can increase the value of the  node to increase the amount the shader displaces the vertices by, as shown in the three examples below:

## Control the scale of displacement

Control the scale at which the shader offsets the vertices by multiplying the position by a constant factor before sampling the  node:

The following image shows the interior of the Displacement node graph:

![A screenshot of the interior of the Displacement node graph described above. A uniform input Float node with the name Amplitude node connects to a Convert node, which connects to the Amplitude input of a Noise 3D node. A uniform input Vector3 (Float) node with the name Position connects to the top input of a Multiply node. A uniform input Float node with the name Scale connects to the bottom input of the same Multiply node. The output of the Multiply node connects to the Position input of the Noise 3D node. The Out output of the Noise 3D node connects to the Model Position Offset input of the Outputs node.](./../../assets/vertex-displacement-6.png)

You can increase the value of the  node to increase the scale at which the shader displaces the vertices, as shown in the three examples below:

## Displace the vertices over time

Animate the vertices by moving the position over time, following these steps:

The following image shows the interior of the Displacement node graph:

![A screenshot of the interior of the Displacement node graph described above. A uniform input Float node with the name Amplitude node connects to a Convert node, which connects to the Amplitude input of a Noise 3D node. A uniform input Vector3 (Float) node with the name Position connects to the top input of a Multiply node. A uniform input Float node with the name Scale connects to the bottom input of the same Multiply node. The output of the Multiply node connects to the top input of an Add node. A Time node connects to the bottom input of the same Add node. The output of the Add node connects to the Position input of the Noise 3D node. The Out output of the Noise 3D node connects to the Model Position Offset input of the Outputs node.](./../../assets/vertex-displacement-7.png)

The vertices of the mesh move over time, as shown below:

[Video: A video of the Shader Graph preview window rendering a cube mesh. The mesh's vertices move and distort in shape over time.](./../../assets/vertex-displacement-7-1.mp4.mp4)

## Control the rate of displacement

Control the rate at which the shader displaces the vertices over time by multiplying the time by a constant factor:

The following image shows the interior of the Displacement node graph:

![A screenshot of the interior of the Displacement node graph described above. A uniform input Float node with the name Amplitude node connects to a Convert node, which connects to the Amplitude input of a Noise 3D node. A uniform input Vector3 (Float) node with the name Position connects to the top input of a Multiply node. A uniform input Float node with the name Scale connects to the bottom input of the same Multiply node. The output of the Multiply node connects to the top input of an Add node. A uniform input Float node with the name Rate connects to the top input of another Multiply node. A Time node connects to the bottom input of that same Multiply node. The output of that Multiply node connects to the bottom input of the Add node. The output of the Add node connects to the Position input of the Noise 3D node. The Out output of the Noise 3D node connects to the Model Position Offset input of the Outputs node.](./../../assets/vertex-displacement-8.png)

You can adjust the  value to control the speed of the vertex displacement animation, as shown in the three examples below:

## Create uniform inputs

Back in the main graph of your shader, create uniform inputs for each of the input properties of the  node — , , and  — so that you can adjust these properties at the material level:

Creating uniform inputs also allows you to adjust the shader properties in code with the [setParameter(name:value:)](../../https_//developer.apple.com/documentation/realitykit/shadergraphmaterial/setparameter(name_value_)/-------setparameter(name_value_).md) method, for example.

## Correct the normal directions

When displacing the vertices of a mesh, you may notice that the way the mesh reacts to lighting no longer appears correct. This is typically the result of displacing the vertex positions without also modifying the vertex normals to match. For an obvious example, apply the material to the plane mesh in the preview window and observe how flat it looks despite its vertices moving:

[Video: A video of the Shader Graph preview window rendering a plane mesh. The mesh's vertices move and distort in shape over time, but the lighting appears uniform across the entire surface in a way that doesn't look correct.](./../../assets/vertex-displacement-11.mp4.mp4)

See [Correcting normals after vertex displacement with Shader Graph](../../realitycomposerpro/correcting-normals-after-vertex-displacement-with-shader-graph/--correcting-normals-after-vertex-displacement-with-shader-graph.md) for information on how you can approximate and correct the normal directions, as shown in the following video:

[Video: A video of the Shader Graph preview window rendering a plane mesh. The mesh's vertices move and distort in shape over time, and the lighting appears to correctly respond to the distorted shape of the surface.](./../../assets/vertex-displacement-12.mp4.mp4)

