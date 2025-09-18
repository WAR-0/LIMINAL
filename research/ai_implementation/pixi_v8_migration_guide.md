# v8 Migration Guide

Welcome to the PixiJS v8 Migration Guide! This document is designed to help you smoothly transition your projects from PixiJS v7 to the latest and greatest PixiJS v8. Please follow these steps to ensure a successful migration.

## Table of Contents[​]()

1.  [Introduction]()
2.  [Breaking Changes]()
3.  [Deprecated Features]()
4.  [Resources]()

PixiJS v8 introduces several exciting changes and improvements that dramatically enhance the performance of the renderer. While we've made efforts to keep the migration process as smooth as possible, some breaking changes are inevitable. This guide will walk you through the necessary steps to migrate your PixiJS v7 project to PixiJS v8.

Before diving into the migration process, let's review the breaking changes introduced in PixiJS v8. Make sure to pay close attention to these changes as they may impact your existing codebase.

Generally, the answer is yes! But currently, there may be reasons that suggest it's best not to upgrade just yet. Ask yourself the following question:

*   **Does your project leverage existing Pixi libraries that have not yet been migrated to v8?** We are working hard to migrate our key libraries to v8 but did not want this to be a blocker for those who are using pure Pixi. This means some libraries will not have a v8 counterpart just yet. It's best to hold off on migration if this is the case for you.

**Migrated**

*   Filters
*   Sound
*   Gif
*   Storybook
*   UI
*   Open Games

**Migrating Right Now:**

*   React
*   Spine (esoteric version)

**To Be Migrated:**

*   Pixi layers (rather than migrating this, we will likely incorporate it directly into PixiJS v8 as a feature)

Since version 5, PixiJS has utilized individual sub-packages to organize its codebase into smaller units. However, this approach led to issues, such as conflicting installations of different PixiJS versions, causing complications with internal caches.

In v8, PixiJS has reverted to a single-package structure. While you can still import specific parts of PixiJS, you only need to install the main package.

**Old:**

    import { Application } from '@pixi/app';import { Sprite } from '@pixi/sprite';

**New:**

    import { Application, Sprite } from 'pixi.js';

PixiJS uses an "extensions" system to add renderer functionality. By default, PixiJS includes many extensions for a comprehensive out-of-the-box experience. However, for full control over features and bundle size, you can manually import specific PixiJS components.

    // imported by defaultimport 'pixi.js/accessibility';import 'pixi.js/app';import 'pixi.js/events';import 'pixi.js/filters';import 'pixi.js/sprite-tiling';import 'pixi.js/text';import 'pixi.js/text-bitmap';import 'pixi.js/text-html';import 'pixi.js/graphics';import 'pixi.js/mesh';import 'pixi.js/sprite-nine-slice';// not added by default, everyone needs to import these manuallyimport 'pixi.js/advanced-blend-modes';import 'pixi.js/unsafe-eval';import 'pixi.js/prepare';import 'pixi.js/math-extras';import 'pixi.js/dds';import 'pixi.js/ktx';import 'pixi.js/ktx2';import 'pixi.js/basis';import { Application } from 'pixi.js';const app = new Application();await app.init({  manageImports: false, // disable importing the above extensions});

When initializing the application, you can disable the auto-import feature, preventing PixiJS from importing any extensions automatically. You'll need to import them manually, as demonstrated above.

It should also be noted that the `pixi.js/text-bitmap`, also add `Assets` loading functionality. Therefore if you want to load bitmap fonts **BEFORE** initialising the renderer, you will need to import this extension.

    import 'pixi.js/text-bitmap';import { Assets, Application } from 'pixi.js';await Assets.load('my-font.fnt'); // If 'pixi.js/text-bitmap' is not imported, this will not loadawait new Application().init();

PixiJS will now need to be initialised asynchronously. With the introduction of the WebGPU renderer PixiJS will now need to be awaited before being used

**Old:**

    import { Application } from 'pixi.js';const app = new Application();// do pixi things

**New:**

    import { Application } from 'pixi.js';const app = new Application();(async () => {  await app.init({    // application options  });  // do pixi things})();

With this change it also means that the `ApplicationOptions` object can now be passed into the `init` function instead of the constructor.

Textures structures have been modified to simplify what was becoming quite a mess behind the scenes in v7. Textures no longer know or manage loading of resources. This needs to be done upfront by you or the assets manager. Textures expect full loaded resources only. This makes things so much easier to manage as the validation of a texture can essentially be done at construction time and left at that! BaseTexture no longer exists. In stead we now have a variety of TextureSources available. A texture source combines the settings of a texture with how to upload and use that texture. In v8 there are the following texture sources:

TextureSource - a vanilla texture that you can render too or upload however you wish. (used mainly by render textures) ImageSource - a texture source that contains an image resource of some kind (eg ImageBitmap or html image) CanvasSource - a canvas source that contains a canvas. Used mainly for rendering canvases or rendering to a canvas (webGPU) VideoSource - a texture source that contains a video. Takes care of updating the texture on the GPU to ensure that they stay in sync. BufferSource - a texture source that contains a buffer. What ever you want really! make sure your buffer type and format are compatible! CompressedSource - a texture source that handles compressed textures. Used by the GPU compressed texture formats.

Whilst the majority of the time `Assets` will return Textures you may want to make you own! More power to ya!

To create a texture source the signature differs from baseTexture. example:

    const image = new Image();image.onload = function(){  // create a texture source  const source = new ImageSource({    resource: image,  });  // create a texture  const texture = new Texture({    source  });}image.src = 'myImage.png';

There are a few key changes to the Graphics API. In fact this is probably the most changed part of v8. We have added deprecations where possible but below is the rundown of changes:

*   Instead of beginning a fill or a stroke and then building a shape, v8 asks you to build your shape and then stroke / fill it. The terminology of `Line` has been replaced with the terminology of `Stroke`

**Old:**

    // red rectconst graphics = new Graphics().beginFill(0xff0000).drawRect(50, 50, 100, 100).endFill();// blue rect with strokeconst graphics2 = new Graphics().lineStyle(2, 'white').beginFill('blue').circle(530, 50, 140, 100).endFill();

**New:**

    // red rectconst graphics = new Graphics().rect(50, 50, 100, 100).fill(0xff0000);// blue rect with strokeconst graphics2 = new Graphics().rect(50, 50, 100, 100).fill('blue').stroke({ width: 2, color: 'white' });

*   Shape functions have been renamed. Each drawing function has been simplified into a shorter version of its name. They have the same parameters though:

| v7 API Call | v8 API Equivalent |
| --- | --- |
| drawChamferRect | chamferRect |
| drawCircle | circle |
| drawEllipse | ellipse |
| drawFilletRect | filletRect |
| drawPolygon | poly |
| drawRect | rect |
| drawRegularPolygon | regularPoly |
| drawRoundedPolygon | roundPoly |
| drawRoundedRect | roundRect |
| drawRoundedShape | roundShape |
| drawStar | star |

*   fills functions expect `FillStyle` options or a color, rather than a string of parameters. This also replaces `beginTextureFill`

**Old:**

    const rect = new Graphics()  .beginTextureFill({ texture: Texture.WHITE, alpha: 0.5, color: 0xff0000 })  .drawRect(0, 0, 100, 100)  .endFill()  .beginFill(0xffff00, 0.5)  .drawRect(100, 0, 100, 100)  .endFill();

**New:**

    const rect = new Graphics()  .rect(0, 0, 100, 100)  .fill({ texture: Texture.WHITE, alpha: 0.5, color: 0xff0000 })  .rect(100, 0, 100, 100)  .fill({ color: 0xffff00, alpha: 0.5 });

*   Strokes now expect a `StrokeStyle` object rather than a string of parameters. This also replaces `lineTextureStyle`

**Old:**

    const graphics = new Graphics()  .lineStyle(10, 0xff0000, 1, 0.5, true)  .moveTo(0, 0)  .lineTo(100, 100);const graphics2 = new Graphics()  .lineTextureStyle({ texture: Texture.WHITE, width: 10, color: 0xff0000 })  .moveTo(0, 0)  .lineTo(100, 100);

**New:**

    const graphics = new Graphics()  .moveTo(0, 0)  .lineTo(100, 100)  .stroke({ width: 10, color: 0xff0000, alpha: 1, alignment: 0.5, cap: 'round' });const graphics2 = new Graphics()  .moveTo(0, 0)  .lineTo(100, 100)  .stroke({ texture: Texture.WHITE, width: 10, color: 0xff0000 });

*   `drawHole` has been removed. Instead, you can now use `cut` to create holes in your shapes.

**Old:**

    const graphics = new Graphics()  .beginFill(0xff0000)  .drawRect(0, 0, 100, 100)  .beginHole()  .drawRect(25, 25, 50, 50)  .endHole()  .endFill();

**New:**

    const graphics = new Graphics()  .rect(0, 0, 100, 100)  .cut()  .rect(25, 25, 50, 50)  .fill(0xff0000);

*   `clear` has been renamed to `destroy` and now has a different function. To clear a graphic, you should now use `clear`.

**Old:**

    const graphics = new Graphics();graphics.beginFill(0xff0000).drawRect(0, 0, 100, 100).endFill();graphics.clear(); // this would clear the graphic

**New:**

    const graphics = new Graphics();graphics.rect(0, 0, 100, 100).fill(0xff0000);graphics.clear(); // this will clear the graphic

*   `geometry.invalidate` has been removed. The geometry will now automatically invalidate itself when you modify it.

*   `closePath` has been removed. Instead, you can now use the `close` function on the `GraphicsPath` object.

**Old:**

    const graphics = new Graphics().lineStyle(2, 0xffffff).moveTo(0, 0).lineTo(50, 0).lineTo(50, 50).closePath();

**New:**

    const graphics = new Graphics().path.moveTo(0, 0).lineTo(50, 0).lineTo(50, 50).close();new Graphics().path(graphics.path).stroke({ width: 2, color: 0xffffff });

*   `getCanvas` has been removed. Instead, you can now use the `getCanvas` function on the `Renderer`.

**Old:**

    const canvas = graphics.getCanvas();

**New:**

    const canvas = renderer.canvas.getCanvas(graphics);

*   `generateCanvasTexture` has been removed. Instead, you can now use the `generateTexture` function on the `Renderer`.

**Old:**

    const texture = graphics.generateCanvasTexture();

**New:**

    const texture = renderer.generateTexture(graphics);

*   `isMask` has been removed. Instead, you can now use the `isMask` property on the `Graphics` object.

**Old:**

    graphics.isMask = true;

**New:**

    graphics.isMask = true;

*   `containsPoint` has been removed. Instead, you can now use the `containsPoint` function on the `Graphics` object.

**Old:**

    const contains = graphics.containsPoint(new Point(0, 0));

**New:**

    const contains = graphics.containsPoint({ x: 0, y: 0 });

*   `adaptive` property has been removed. Instead, you can now use the `adaptive` property on the `TextStyle` object.

**Old:**

    const text = new Text('Hello World', { adaptive: true });

**New:**

    const text = new Text({ text: 'Hello World', style: { adaptive: true } });

*   `lineStyle` `native` parameter has been removed. Instead, you can now use the `cap`, `join`, and `miterLimit` properties on the `StrokeStyle` object.

**Old:**

    const graphics = new Graphics().lineStyle(10, 0xffffff, 1, 0.5, false, 'round', 'round', 10);

**New:**

    const graphics = new Graphics().stroke({ width: 10, color: 0xffffff, alpha: 1, alignment: 0.5, cap: 'round', join: 'round', miterLimit: 10 });

Shaders have been completely rewritten in v8. The new shader system is more powerful and flexible, but it also means that your old shaders will need to be updated. We have tried to make this as easy as possible by providing a compatibility layer that will allow you to use your old shaders with the new system. However, we recommend that you update your shaders to the new system as soon as possible.

**Old:**

    const fragment = `varying vec2 vTextureCoord;uniform sampler2D uSampler;void main(void){   gl_FragColor = texture2D(uSampler, vTextureCoord);}`;const myFilter = new PIXI.Filter(null, fragment);

**New:**

    const fragment = `in vec2 vTextureCoord;out vec4 fragColor;uniform sampler2D uSampler;void main(void){   fragColor = texture(uSampler, vTextureCoord);}`;const myFilter = new PIXI.Filter({   fragment,});

For more information on the new shader system, please see the [Shader]() documentation.

Filters have been completely rewritten in v8. The new filter system is more powerful and flexible, but it also means that your old filters will need to be updated. We have tried to make this as easy as possible by providing a compatibility layer that will allow you to use your old filters with the new system. However, we recommend that you update your filters to the new system as soon as possible.

**Old:**

    const myFilter = new PIXI.Filter(null, fragment);myFilter.uniforms.myUniform = 1.0;

**New:**

    const myFilter = new PIXI.Filter({   fragment,   uniforms: {      myUniform: 1.0,   },});

For more information on the new filter system, please see the [Filter]() documentation.

`ParticleContainer` has been renamed to `ParticleContainer` and has been moved to the `pixi.js/particle-container` package.

**Old:**

    import { ParticleContainer } from '@pixi/particle-container';const particleContainer = new ParticleContainer();

**New:**

    import { ParticleContainer } from 'pixi.js/particle-container';const particleContainer = new ParticleContainer();

*   `DisplayObject` has been removed. `Container` is now the base class for all PixiJS objects.
*   `updateTransform` has been removed as nodes no longer have a transform. Instead, you can use the `localTransform` and `worldTransform` properties.
*   `setTransform` has been removed. Instead, you can use the `position`, `scale`, and `rotation` properties.
*   `setFromMatrix` has been removed. Instead, you can use the `setMatrix` function.
*   `getBounds` has been removed. Instead, you can use the `getBounds` function on the `Bounds` object.
*   `getLocalBounds` has been removed. Instead, you can use the `getLocalBounds` function on the `Bounds` object.
*   `toGlobal` has been removed. Instead, you can use the `toGlobal` function on the `Transform` object.
*   `toLocal` has been removed. Instead, you can use the `toLocal` function on the `Transform` object.
*   `renderWebGL` has been removed. Instead, you can use the `render` function on the `Renderer` object.
*   `renderCanvas` has been removed. Instead, you can use the `render` function on the `Renderer` object.
*   `calculateVertices` has been removed. Instead, you can use the `calculateVertices` function on the `Geometry` object.
*   `calculateUvs` has been removed. Instead, you can use the `calculateUvs` function on the `Geometry` object.
*   `_calculateBounds` has been removed. Instead, you can use the `calculateBounds` function on the `Bounds` object.
*   `container.renderable` has been removed. Instead, you can use the `renderable` property on the `Container` object.
*   `container.visible` has been removed. Instead, you can use the `visible` property on the `Container` object.
*   `container.alpha` has been removed. Instead, you can use the `alpha` property on the `Container` object.
*   `container.worldAlpha` has been removed. Instead, you can use the `worldAlpha` property on the `Container` object.
*   `container.interactive` has been removed. Instead, you can use the `interactive` property on the `Container` object.
*   `container.interactiveChildren` has been removed. Instead, you can use the `interactiveChildren` property on the `Container` object.
*   `container.hitArea` has been removed. Instead, you can use the `hitArea` property on the `Container` object.
*   `container.buttonMode` has been removed. Instead, you can use the `buttonMode` property on the `Container` object.
*   `container.cursor` has been removed. Instead, you can use the `cursor` property on the `Container` object.
*   `container.trackedPointers` has been removed. Instead, you can use the `trackedPointers` property on the `Container` object.
*   `container.render` has been removed. Instead, you can use the `render` function on the `Renderer` object.
*   `container.destroy` has been removed. Instead, you can use the `destroy` function on the `Container` object.

PixiJS v8 deprecates several features that are no longer recommended for use. These features will be removed in a future version of PixiJS, so it is recommended that you migrate away from them as soon as possible.

*   `PIXI.loaders.Loader` is deprecated. Instead, you should use the `Assets` class.
*   `PIXI.Application.renderer` is deprecated. Instead, you should use the `renderer` property on the `Application` object.
*   `PIXI.Application.stage` is deprecated. Instead, you should use the `stage` property on the `Application` object.
*   `PIXI.Application.ticker` is deprecated. Instead, you should use the `ticker` property on the `Application` object.
*   `PIXI.Application.view` is deprecated. Instead, you should use the `view` property on the `Application` object.
*   `PIXI.Graphics.drawShape` is deprecated. Instead, you should use the `draw` function on the `Graphics` object.
*   `PIXI.Graphics.drawPolygon` is deprecated. Instead, you should use the `poly` function on the `Graphics` object.
*   `PIXI.Graphics.drawRect` is deprecated. Instead, you should use the `rect` function on the `Graphics` object.
*   `PIXI.Graphics.drawRoundedRect` is deprecated. Instead, you should use the `roundRect` function on the `Graphics` object.
*   `PIXI.Graphics.drawCircle` is deprecated. Instead, you should use the `circle` function on the `Graphics` object.
*   `PIXI.Graphics.drawEllipse` is deprecated. Instead, you should use the `ellipse` function on the `Graphics` object.
*   `PIXI.Graphics.drawStar` is deprecated. Instead, you should use the `star` function on the `Graphics` object.
*   `PIXI.Graphics.drawChamferRect` is deprecated. Instead, you should use the `chamferRect` function on the `Graphics` object.
*   `PIXI.Graphics.drawFilletRect` is deprecated. Instead, you should use the `filletRect` function on the `Graphics` object.
*   `PIXI.Graphics.drawRegularPolygon` is deprecated. Instead, you should use the `regularPoly` function on the `Graphics` object.
*   `PIXI.Graphics.drawRoundedPolygon` is deprecated. Instead, you should use the `roundPoly` function on the `Graphics` object.
*   `PIXI.Graphics.drawRoundedShape` is deprecated. Instead, you should use the `roundShape` function on the `Graphics` object.
*   `PIXI.Graphics.lineStyle` is deprecated. Instead, you should use the `stroke` function on the `Graphics` object.
*   `PIXI.Graphics.beginFill` is deprecated. Instead, you should use the `fill` function on the `Graphics` in conjunction with a shape.
*   `PIXI.Graphics.endFill` is deprecated. Instead, you should use the `fill` function on the `Graphics` in conjunction with a shape.
*   `PIXI.Graphics.beginTextureFill` is deprecated. Instead, you should use the `fill` function on the `Graphics` in conjunction with a shape.
*   `PIXI.Graphics.lineTextureStyle` is deprecated. Instead, you should use the `stroke` function on the `Graphics` in conjunction with a shape.
*   `PIXI.Graphics.clear` is deprecated. Instead, you should use the `clear` function on the `Graphics` object.
*   `PIXI.Graphics.geometry.invalidate` is deprecated. The geometry will now automatically invalidate itself when you modify it.
*   `PIXI.Graphics.closePath` is deprecated. Instead, you can now use the `close` function on the `GraphicsPath` object.
*   `PIXI.Graphics.getCanvas` is deprecated. Instead, you can now use the `getCanvas` function on the `Renderer`.
*   `PIXI.Graphics.generateCanvasTexture` is deprecated. Instead, you can now use the `generateTexture` function on the `Renderer`.
*   `PIXI.Graphics.isMask` is deprecated. Instead, you can now use the `isMask` property on the `Graphics` object.
*   `PIXI.Graphics.containsPoint` is deprecated. Instead, you can now use the `containsPoint` function on the `Graphics` object.
*   `PIXI.Text.adaptive` is deprecated. Instead, you can now use the `adaptive` property on the `TextStyle` object.
*   `PIXI.Text.lineStyle` `native` parameter is deprecated. Instead, you can now use the `cap`, `join`, and `miterLimit` properties on the `StrokeStyle` object.

*   [PixiJS v8 Migration Guide on GitHub](https://github.com/pixijs/pixijs/discussions/9791)
*   [PixiJS v8 API Documentation](https://pixijs.download/v8.0.0-rc.2/docs/)
*   [PixiJS v8 Examples](https://pixijs.io/examples-v8/)
*   [PixiJS Discord Server](https://discord.gg/pixijs)

We hope this migration guide has been helpful. If you have any questions or feedback, please don't hesitate to reach out to us on the PixiJS Discord server.


