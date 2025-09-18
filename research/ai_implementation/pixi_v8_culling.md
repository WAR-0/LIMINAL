# Optimizing Rendering with PixiJS v8: A Deep Dive into the New Culling API

The culling feature has been available as an extension in earlier versions of PixiJS, but with the release of PixiJS v8, it is now officially integrated into the core. This motivated us to revisit the feature and explore how to take advantage of it. However, the current documentation and examples are still limited. In this article, I’ll walk through how culling works in PixiJS v8, highlight some nuances, and share my personal insights on its usage.

## Understanding the Intuition

In our project, we often render a long list of UI items, which are masked with a rectangular shape. Performance tends to degrade when rendering 500+ items, especially on lower-end devices. Currently, we manually set `.visible = false` for any object outside the visible screen. We wanted to explore if culling could help us automate this process efficiently.

### First Look at PixiJS v8 Culling

Both the [v8 Migration Guide](https://pixijs.com/8.x/guides/migrations/v8) and [Performance Tips](https://pixijs.com/8.x/guides/concepts/performance-tips) mention the new culling API, but they lack detailed usage examples. Most tutorials online still reference older versions. Fortunately, the [Culler test script](https://github.com/pixijs/pixijs/blob/dev/tests/culling/Culler.test.ts) proved to be an invaluable resource in understanding how the feature is intended to work.

From the migration guide, here’s a basic usage example:

```javascript
const container = new Container();
const view = new Rectangle(0, 0, 800, 600);
container.cullable = true;
container.cullArea = new Rectangle(0, 0, 400, 400);
container.cullableChildren = false;
app.stage.addChild(container);
Culler.shared.cull(container, view);
```

### Defining the Cull View

*   **The view** should be defined as a `Rectangle` using global coordinates relative to the canvas.
*   If the canvas size is dynamic (e.g., responsive layout), you should update the `view` accordingly and invoke `Culler.shared.cull()` again. For example:

```javascript
let timeSinceLoad = 0;
app.ticker.add(function (ticker) {
  timeSinceLoad += ticker.deltaMS;
  view = new Rectangle(
    app.renderer.width / 2 - 100,
    app.renderer.height / 2 - 100,
    200 + timeSinceLoad * 0.1,
    200 + timeSinceLoad * 0.1
  );
  Culler.shared.cull(container, view, false);
});
```

*   In most cases, you can use `app.stage` and `app.screen` directly as inputs for culling. This works well in a game loop or render tick:

```javascript
app.ticker.add(() => {
  Culler.shared.cull(app.stage, app.screen);
});
```

## Understanding Cull Area

The `cullArea` property can be confusing. It’s critical to note that this rectangle is defined in global coordinates, and does not inherit transformations like position or rotation from the container it’s attached to. This can lead to unexpected behavior, as seen in one of the official test cases:

```javascript
const view = { x: 0, y: 0, width: 100, height: 100 };
it("cullable container with cullArea should not be rendered if the bounds do not intersect the frame", () =>
{
    const container = new Container();
    const graphics = container.addChild(new Graphics().rect(0, 0, 10, 10).fill());
    container.cullable = true;
    container.cullArea = new Rectangle(-10, -10, 10, 10);
    container.x = container.y = 107.08;
    container.rotation = Math.PI / 4;
    Culler.shared.cull(container, view, false);
    expect(container.culled).toBe(true);
    expect(graphics.culled).toBe(false);
});
```

Even though the container and its child graphics are visually inside the view, the `cullArea` (defined in global space) is outside the `view`. As a result, the container is culled, but its graphics child is not.

> Although `graphics.culled` is `false` here, it is not rendered because its parent `container` has been culled.

In practice, I find it rare to define a custom `cullArea` that doesn’t align with the container’s own bounds. However, there may be edge cases—like complex layering or shared masks—where this flexibility becomes useful.

## Conclusion

While the built-in culling system in PixiJS v8 simplifies object visibility in static or fixed-size canvases, it adds a bit of complexity when dealing with responsive layouts or dynamically changing viewports. You must ensure that the view and cull areas are accurately maintained in global space.

In our case, sticking with [manual masking](https://www.richardfu.net/optimizing-scalable-ui-elements-with-pixi-js-ninesliceplane/) combined with setting `.visible = false` based on Y-position checks remains the most straightforward and performant approach for large lists of items:  
`if (y < -object.height / 2 || y > screen.height + object.height / 2) object.visible = false;`

That said, Pixi’s new culling API is a powerful addition—especially for dynamic scenes—and it’s worth exploring to see if it fits your rendering optimization needs.


