> [REALITYCOMPOSERPRO](../../realitycomposerpro/-realitycomposerpro.md) > [CORRECTING NORMALS AFTER VERTEX DISPLACEMENT WITH SHADER GRAPH](../../realitycomposerpro/correcting-normals-after-vertex-displacement-with-shader-graph/--correcting-normals-after-vertex-displacement-with-shader-graph.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../realitykit/-realitykit.md)
- [🛠 Reality Composer Pro](../../realitycomposerpro/-realitycomposerpro.md)
- [🕶 visionOS Home](../../visionos/-visionos.md)

---

# Correcting normals after vertex displacement with Shader Graph

Approximate new vertex normals for materials that displace a mesh’s vertices.

## Overview

Vertex displacement shaders can create a myriad of dynamic effects, such as water, terrain, and undulating surfaces. However, displacing vertex positions without correcting the vertex normals can result in incorrect lighting, as shown in the examples below:

Assuming the vertices of your mesh have correct normals predisplacement, and the displacement is a continuous function of the vertex positions in model space, you can correct the normals of your vertex displacement shader by sampling the displacement of neighboring points along the surface of the mesh and approximating a new normal direction with the cross product.

For each vertex in the mesh, find two neighboring points on the surface of the mesh that are in orthogonal directions to one another. Approximate the new normal direction by displacing these points with the same function that displaces the original vertex, and then take the cross product between the new vectors pointing toward them. See [Loading entities with ShaderGraph materials](../../realitycomposerpro/loading-entities-with-shadergraph-materials/--loading-entities-with-shadergraph-materials.md) to download a sample project containing this shader along with many others.

## Create a displacement node graph

Start by creating a node graph that takes a position input in  or  space and outputs a model position offset vector. See [Creating a vertex displacement material with Shader Graph](../../realitycomposerpro/creating-a-vertex-displacement-material-with-shader-graph/--creating-a-vertex-displacement-material-with-shader-graph.md) for one such example, replicated below:

![A screenshot of the vertex displacement Shader Graph the aforementioned article describes. Three uniform input Float nodes, Amplitude, Frequency, and Rate, connect to their respective inputs on the Displacement node graph node: Amplitude, Frequency, and Rate. A Position node connects to the Position input of the Displacement node graph. The Model Position Offset output of the Displacement node graph connects to the Model Position Offset input of a Geometry Modifier node, whose output connects to the Custom Geometry Modifier input of the Outputs node. The output of a Preview Surface node connects to the Custom Surface input of the Outputs node.](./../../assets/correcting-normals-3.png)

In this example, the Displacement node graph is as follows:

![A screenshot of the contents of the Displacement node graph described above. A uniform input Float node with the name Amplitude node connects to a Convert node, which connects to the Amplitude input of a Noise 3D node. A uniform input Vector3 (Float) node with the name Position connects to the top input of a Multiply node. A uniform input Float node with the name Frequency connects to the bottom input of the same Multiply node. The output of the Multiply node connects to the top input of an Add node. A uniform input Float node with the name Rate connects to the top input of another Multiply node. A Time node connects to the bottom input of that same Multiply node. The output of that Multiply node connects to the bottom input of the Add node. The output of the Add node connects to the Position input of the Noise 3D node. The Out output of the Noise 3D node connects to the Model Position Offset input of the Outputs node.](./../../assets/correcting-normals-4.png)

## Set up the offset and normal node graph

Create the outline for a node graph that calculates the model position offset and the new normal direction in tandem:

The main body of the shader is shown here:

![A screenshot of the Shader Graph described above. Three uniform input Float nodes, Amplitude, Frequency, and Rate, connect to their respective inputs on the OffsetAndNormal node graph node: Amplitude, Frequency, and Rate. The Model Position Offset and Normal outputs of the OffsetAndNormal node graph connect to their respective inputs on a Geometry Modifier node: Model Position Offset and Normal. The output of the Geometry Modifier node connects to the Custom Geometry Modifier input of the Outputs node. The output of a Preview Surface node connects to the Custom Surface input of the Outputs node.](./../../assets/correcting-normals-5.png)

See the interior of the OffsetAndNormal node graph below:

![A screenshot of the contents of the OffsetAndNormal node graph described above. Three uniform input Float nodes, Amplitude, Frequency, and Rate, connect to their respective inputs on the Displacement node graph node: Amplitude, Frequency, and Rate. A Position node connects to the Position input of the Displacement node graph. The Model Position Offset output of the Displacement node graph connects to the Model Position Offset input of the Outputs node.](./../../assets/correcting-normals-6.png)

## Set up the neighbors node graph

Prepare a new node graph to calculate two orthogonal neighboring positions at a given distance away from a given vertex position:

The interior of the OffsetAndNormal node graph is as follows:

![A screenshot of the contents of the OffsetAndNormal node graph described above. Three uniform input Float nodes, Amplitude, Frequency, and Rate, connect to their respective inputs on the Displacement node graph node: Amplitude, Frequency, and Rate. A Position node connects to the Position input of the Displacement node graph, as well as the Position input of a Neighbors node graph. The Model Position Offset output of the Displacement node graph connects to the Model Position Offset input of the Outputs node.](./../../assets/correcting-normals-7.png)

## Find neighboring positions along the surface

Calculate two orthogonal neighbor positions inside the Neighbors node graph:

The interior of the Neighbors node graph is as follows:

![A screenshot of the Shader Graph described above. A Normal node connects to the top input of a Cross Product node with the name Tangent. The bottom input of the Tangent Cross Product node is the vector (1.00, 0.00, 0.00). The output of the Tangent Cross Product node connects to the top input of a Cross Product node with the name Bitangent. The Normal node connects to the bottom input of the Bitangent Cross Product node. The outputs of the Tangent and Bitangent Cross Product nodes connect to Normalize nodes, which connect to the top input of Multiply nodes, whose bottom input comes from a uniform input Float node with the name Distance. The outputs of the Multiply nodes connect to the bottom inputs of two Add nodes, whose top input comes from a uniform input Vector3 (Float) node with the name Position. The output of the Add nodes connect to the Neighbor1Position and Neighbor2Position inputs of the Outputs node.](./../../assets/correcting-normals-8.png)

While this node graph outputs orthogonal neighboring positions under most circumstances, it fails when the normal vector is similar or equal to the arbitrary unit vector you take the cross product with to get the tangent. One approach to remedying this issue is to compute the similarity between the normal and the arbitrary unit vector with a dot product, and then linearly interpolate toward a different arbitrary unit vector by the similarity value before computing the first tangent, as shown here:

![A screenshot of the Shader Graph described above. A Normal node connects to the bottom input of a Dot Product node with the name Similarity. The top input of the Dot Product node is a Vector3 (Float) node with the name ArbitraryVector1 and a value of (1.00, 0.00, 0.00). The output of the Similarity Dot Product node connects to an Abs node, which connects to the Mix input of a Mix node. ArbitraryVector1 connects to the Background input of the Mix node, while a Vector3 (Float) node with the name ArbitraryVector2 and a value of (0.00, 1.00, 0.00) connects to the Foreground input. The Mix node connects to the top input of a Cross Product node with the name Tangent. The bottom input of the Tangent Cross Product node is the vector (1.00, 0.00, 0.00). The output of the Tangent Cross Product node connects to the top input of a Cross Product node with the name Bitangent. The Normal node connects to the bottom input of the Bitangent Cross Product node. The outputs of the Tangent and Bitangent Cross Product nodes connect to Normalize nodes, which connect to the top input of Multiply nodes, whose bottom input comes from a uniform input Float node with the name Distance. The outputs of the Multiply nodes connect to the bottom inputs of two Add nodes, whose top input comes from a uniform input Vector3 (Float) node with the name Position. The output of the Add nodes connect to the Neighbor1Position and Neighbor2Position inputs of the Outputs node.](./../../assets/correcting-normals-9.png)

## Sample the displacement at the neighboring positions

Calculate the model position offset at both of the neighboring positions by making instances of your Displacement node graph:

Perform these steps inside the OffsetAndNormal node graph, connecting the Amplitude, Rate, and Scale input nodes to the corresponding inputs of each Displacement node, as shown in the image below:

![A screenshot of the contents of the OffsetAndNormal node graph described above. Three uniform input Float nodes, Amplitude, Frequency, and Rate, connect to their respective inputs on the Displacement node graph node and its two instances: DisplacementNeighbor1 and DisplacementNeighbor2. A Position node connects to the Position input of the Displacement node graph, as well as the Position input of a Neighbors node graph. The Neighbor1 Position output of the Neighbors node connects to the Position input of the DisplacementNeighbor1 node graph, while the Neighbor2 Position output connects to the Position input of the DisplacementNeighbor2 node graph. The Model Position Offset output of the Displacement node graph connects to the Model Position Offset input of the Outputs node.](./../../assets/correcting-normals-10.png)

## Compute the new normal direction

Calculate the new normal direction by taking the cross product between the directions from the displaced vertex position to the displaced neighbor positions:

The resulting node graph is as follows:

![A screenshot of the contents of the OffsetAndNormal node graph described above. Three uniform input Float nodes, Amplitude, Frequency, and Rate, connect to their respective inputs on the Displacement node graph node and its two instances: DisplacementNeighbor1 and DisplacementNeighbor2. A Position node connects to the Position input of the Displacement node graph, as well as the Position input of a Neighbors node graph. The Neighbor1Position output of the Neighbors node connects to the Position input of the DisplacementNeighbor1 node graph, while the Neighbor2Position output connects to the Position input of the DisplacementNeighbor2 node graph. The value connecting to the Position input of each of the three Displacement node graphs connects to the top input of three Add nodes, whose bottom input comes from the Model Position Offset output of their respective Displacement node graph. The output of the two Add nodes, which connect to the DisplacementNeighbor node graphs, connect to the top input of two Subtract nodes, whose bottom input comes from the output of the Add node that connects to the original Displacement node graph. The output of the Subtract nodes connect to the top and bottom input of a Cross Product node, which connects to a Normalize node, which in turn connects to the Normal input of the Outputs node. The Model Position Offset output of the Displacement node graph connects to the Model Position Offset input of the Outputs node.](./../../assets/correcting-normals-11.png)

The following videos show the result of correcting the normals for a shader displacing the vertices of a plane mesh:

