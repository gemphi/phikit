> [REALITYCOMPOSERPRO](../../realitycomposerpro/-realitycomposerpro.md) > [CONFIGURE THE REALITY COMPOSER PRO PROJECT WINDOW](../../realitycomposerpro/configure-the-reality-composer-pro-project-window/--configure-the-reality-composer-pro-project-window.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../realitykit/-realitykit.md)
- [🛠 Reality Composer Pro](../../realitycomposerpro/-realitycomposerpro.md)
- [🕶 visionOS Home](../../visionos/-visionos.md)

---

# Configuring the Reality Composer Pro project window

Change the appearance of the Reality Composer Pro project window by showing and hiding views, and learn to navigate your content.

## Overview

Reality Composer Pro’s project window is the main interface for viewing, editing, creating and arranging 3D content for your Xcode project using RealityKit. With Xcode open, select XCode > Open Developer Tools > Reality Composer Pro to launch the welcome window.

The welcome window provides three options:

- Create New Project
- Create New Object Capture Model
- Open Existing Project

![A screenshot of the Reality Composer Pro welcome window containing the Reality Composer Pro icon, followed by the title “Reality Composer Pro”, the current version of the software, Create New Project, Create New Object Capture Model, and Open Existing Project.](./../../assets/rcpro-welcomewindow.png)

Reality Composer Pro stores each project file as a  file. To open the project window, you can choose to either create a new project or select an existing  file.

The project window is composed of multiple views:

- The  provides you quick access to actions and settings for your project. In the default toolbar, you can toggle the leading panel, send to device, switch between local space and world space, adjust the viewport camera settings, open the Content Library, and toggle the inspector. You can configure it to fit your project needs by adding and removing tools. To edit the toolbar, control-click the toolbar and select Customize Toolbar.
- The  displays a 3D visual representation of the active scene. This view provides an interactive way to view, select, and manipulate your content from different positions and angles.
- The  shows the hierarchy of all the assets in the current scene. You can add and remove assets, and adjust the relationship of multiple assets in the scene.
- With the , you can view and edit information about the scene or about the selected asset in the navigator or viewport. You can change the values of the components attached to the selected asset and add new components.
- With the , you can manage your current project’s content, build materials for assets with Shader Graph, add timelines for animations, adjust the audio mix of the active scene, and see how the scene performs when running in a RealityKit app.

![A screenshot of a Reality Composer Pro project window. The window has a single tab at the top called Example. The window is divided vertically into two sections. The top section has three views: a navigator view of content in the scene, a viewport in the middle showing a 2D grid, and an inspector on the right. The bottom section has four tabs, labeled Project Browser, Shader Graph, Timelines Audio Mixer, and Statistics. Project Browser is currently selected, and it’s also divided into three sections with a hierarchy on the left, a grid of icons in the middle, and an inspector on the right showing the entity hierarchy for the asset selected in the middle](./../../assets/rcpro-newproject.png)

## Navigate in the viewport

Reality Composer Pro’s viewport provides an interactive way for you to view your project’s assets and scenes in 3D space. You can change how you interact with content and navigate around the scene by selecting a viewport mode. To select a mode, click on the corresponding button located at the bottom-left of the viewport. Additonally, you can change the mode in the menu bar by choosing Viewport and selecting one of the five modes. The viewport toolbar contains a button for Selection Mode, four navigation modes, a Camera Reset, and an Environmental Picker.

![A zoomed in screenshot of Reality Composer Pro’s viewport toolbar containing seven icons: an icon of a mouse cursor surrounded by a circle, a circle with four triangles pointing in each cardinal directions, an equalateral cross with triangles at each tip pointing in each cardinal direction, a circle with a curved crossed pattern to create a 3D sphere, a magnifying glass with a diagonal slash in the center, a camera with a go back style arrow, and a triangle layered on top of another triangle to create two mountains. ](./../../assets/rcpro-viewporttoolbar.png)

The four modes that move the viewport camera around in the scene are:

You can click on one of the viewport mode buttons and drag the cursor around to use the selected viewport mode quickly without having to change your current mode.

[Video: A video that demonstrates how to change the camera movement in Reality Composer Pro's viewport. The viewport contains a tea cup, spoon, and saucer plate in the center. The viewport moves respectively to match the mouse movements.](./../../assets/rcpro-viewportshortcut%401x.mp4.mp4)

### Interact with entites using Selection Mode

The first item in the viewport toolbar is Selection Mode, which allows you to select an entity in the viewport. With an entity selected, you can click on the manipulator and adjust its position, rotation, and scale. The viewpoint camera stays stationary when manipulating an entity.

With Selection Mode, you can use keyboard shortcuts to quickly choose one of the four types of viewport camera movements without having to switch.

You can perform a Marquee select by holding Shift and then clicking and dragging to create a selection box around multiple entities in the scene.

[Video: A video that demonstrates how to select multiple entites in Reality Composer Pro. The viewport contains a tea cup, spoon, and saucer plate in the center. As the mouse moves, a marquee selection box appears and each object is selected. Each object has an outline along each edge as the marquee box vanishes.](./../../assets/rcpro-marquee.mp4.mp4)

If you select an entity that is part of a reference, the root object that hosts the reference is also selected. To select a subentity object that is part of a reference, hold the Control key when clicking on the object.

### Change the viewport environment

You can change the Reality Composer Pro viewport environment to test how your scene looks and sounds in different settings. To access the environment preview picker, select the environment picker icon on the viewport toolbar.

![A screenshot of the viewport with the Kitchen - Day environment selected in the environment preview picker menu. The viewport contains a tea cup, spoon, and saucer plate in the center. Behind the objects is an image of a kitchen scene set during the day. The menu shows four options in a two-by-two grid of rectangles with rounded edges: Studio, Kitchen - Day, Kitchen - Night, and Living Room - Day. There are five options included in the menu: Show Grid, Background, Rotate, Exposure, and Acoustic Environment.](./../../assets/rcpro-environment.png)

The environment preview picker provides some common backgrounds with lighting to use, and offers multiple acoustic environments.

