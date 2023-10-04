const USING_PUSH_CONSTANTS = true;

struct PushConstants {
    viewport: Viewport,
    graphic: Graphic,
}

var<push_constant> push_constants: PushConstants;
