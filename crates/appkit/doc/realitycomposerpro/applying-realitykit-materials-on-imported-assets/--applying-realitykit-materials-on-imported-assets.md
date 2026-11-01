> [REALITYCOMPOSERPRO](../../realitycomposerpro/-realitycomposerpro.md) > [APPHXING REALITYKIT MATERIALS ON IMPORTED ASSETS](../../realitycomposerpro/applying-realitykit-materials-on-imported-assets/--applying-realitykit-materials-on-imported-assets.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../realitykit/-realitykit.md)
- [🛠 Reality Composer Pro](../../realitycomposerpro/-realitycomposerpro.md)
- [🕶 visionOS Home](../../visionos/-visionos.md)

---

# Applying RealityKit materials to your assets

Work with materials in Reality Composer Pro to enhance the appearance of your model.

## Overview

When you import a USDZ model into Reality Composer Pro, it creates a RealityKit material for every material the asset contains. Reality Composer Pro displays any materials in the active scene in the hierarchy view, represented by a paintbrush icon.

![A screenshot of Reality Composer Pro’s navigator view with a USDZ file named cup_saucer_set displayed. In the cup_saucer_set hierarchy, a folder named Looks is opened and a material is highlighted.](./../../assets/rcpro-materialhierarchy.png)

If you select a material in the hierarchy view, you can edit it and adjust its values — such as Roughness, Opacity, and Emissive Color — using the inspector, as shown in the screenshot below.

![A screenshot of a material in Reality Composer Pro’s inspector view. The material contains three sections named References, Material, and Advanced. Material contains a list of the following arguments: Shader, Face Culling, Diffuse Color, Roughness, Metallic, Opacity, and Normal. Advanced contains a list of the following arguments: Clearcoat, Clearcoat Roughness, Emissive Color, Index of Refraction, Ambient Occlusion, and Opacity Threshold.](./../../assets/rcpro-materialinspector-pbr.png)

A material’s inspector view contains three sections: References, Material, and Advanced. The References section lists any entities using the selected material, and allows you to add or remove references. The Material section contains a list of arguments that define the overal appearance of the material.

The Advanced section includes additional arguments you can use to fine-tune certain aspects of the selected material.

If you select a Physically Based Rendering (PBR) material in the hierarchy view, you can edit it using the inspector. You can replace images, colors, or values for any of the PBR attributes with another image, color, or value of your choosing. Any changes you make to a material affect any entity that’s bound to that material. You can also create new materials from scratch by clicking the Add button (+) at the bottom of the scene hierarchy and choosing Material.

