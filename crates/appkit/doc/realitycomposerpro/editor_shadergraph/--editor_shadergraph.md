> [REALITYCOMPOSERPRO](../../realitycomposerpro/-realitycomposerpro.md) > [EDITOR_SHADERGRAPH](../../realitycomposerpro/editor_shadergraph/--editor_shadergraph.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../realitykit/-realitykit.md)
- [🛠 Reality Composer Pro](../../realitycomposerpro/-realitycomposerpro.md)
- [🕶 visionOS Home](../../visionos/-visionos.md)

---

# Designing materials with Shader Graph

Create realistic materials with Shader Graph’s node editor in Reality Composer Pro.

## Overview

You can create two types of materials in Reality Composer Pro:

- [PhysicallyBasedMaterial](../../realitykit/physicallybasedmaterial/--physicallybasedmaterial.md)
- [ShaderGraphMaterial](../../realitykit/shadergraphmaterial/--shadergraphmaterial.md)

Physically Based Rendering (PBR) materials are great for reproducing real-world surfaces. However, they lack the ability to contain logic or represent nonrealistic materials, like cartoon shaders. Shader Graph, on the other hand, provides the tools to create shaders with dynamic logic and stylized effects.

You can build and edit Shader Graph materials using the Shader Graph editor. The Shader Graph editor has nodes that provide you a tremendous amount of control over materials, along with capabilities that would otherwise require writing Metal shaders. To explore all the nodes that are available, see [ShaderGraph](../../shadergraph/-shadergraph.md).

![A screenshot that shows a complex graph of nodes in the Shader Graph tab, creating a 3D material of a mountain landscape. The graph contains three texture nodes containing a texture file, a normals file, and a height map, which are applied to the final output node.](./../../assets/rcpro-shadergraphwindow.png.png)

### Change material attributes using nodes

The materials you build in the editor can affect both the look of an entity and its shape. If you build a node graph and connect it to the Custom Surface pin on the output node, that node graph controls the surface appearance of the model and roughly equates to writing Metal code in a fragment shader. If you build a node graph and connect it to the Custom Geometry Modifier output pin, those nodes control the shape of the entity, which equates to Metal code running in a vertex shader.

Nodes represent values and operations and serve the same purpose as either a variable, constant, or function in Metal. If you need the sine of a value, for example, connect the value’s output node to the input pin of a [Sin](../../shadergraph/math/sin/---sin.md) node. Add new nodes to the graph by double-clicking the background of the Shader Graph view, or click the New Node button on the right side of the screen.

### Update material values at runtime

One of the major benefits of using Shader Graph is the ablility to change values on your custom materials while your app runs. Shader Graph creates , which are parameters you can set and read from Swift to change your material at runtime.

Some examples of how you can use promoted inputs for your materials are:

- If your material has emission that you want to turn on and off, you can create a Boolean input parameter and implement conditional logic based on its value.
- To smoothly interpolate between two colors, you can create a  input parameter and use it to control how to interpolate between the two colors.

You can Control-click on a constant node and select Promote to turn it into a promoted input. You can also turn a promoted input back into a constant by Control-clicking it and selecting Demote in the contextual menu.

To change the value of an input parameter from Swift code, use [setParameter(name:value:)](../../realitykit/shadergraphmaterial/setparameter(name_value_)/---setparameter(name_value_).md), passing the name of the parameter and the new value. Notice that parameter names are case sensitive, so the  string must exactly match what you call the parameter in Shader Graph.

For more on building materials with Shader Graph, refer to the related [Loading entities with ShaderGraph materials](../../realitycomposerpro/loading-entities-with-shadergraph-materials/--loading-entities-with-shadergraph-materials.md). For examples of Shader Graph use in a completed sample, see [Diorama](../../visionos/diorama/--diorama.md) and [Happy Beam](../../visionos/happybeam/--happybeam.md).

