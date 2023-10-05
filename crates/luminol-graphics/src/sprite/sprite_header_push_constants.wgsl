struct PushConstants {
    viewport: Viewport,
    graphic: Graphic,
}

var<push_constant> push_constants: PushConstants;
