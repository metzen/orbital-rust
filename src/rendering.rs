use bevy::camera::visibility::Layer;

pub trait LayerExt {
    const FOREGROUND: Layer;
    const ORBIT: Layer;
    const BACKGROUND: Layer;
    const MAIN: Layer;
    const ATTITUDE_INDICATOR_GRADUATION_LINES: Layer;
    const ATTITUDE_INDICATOR_BORESIGHT_BACKGROUND: Layer;
    const ATTITUDE_INDICATOR_BORESIGHT_FOREGROUND: Layer;
    const ATTITUDE_INDICATOR_VECTOR: Layer;
}

impl LayerExt for Layer {
    const FOREGROUND: Layer = 0;
    const ORBIT: Layer = 1;
    const BACKGROUND: Layer = 2;
    const MAIN: Layer = 3;
    const ATTITUDE_INDICATOR_GRADUATION_LINES: Layer = 4;
    const ATTITUDE_INDICATOR_BORESIGHT_BACKGROUND: Layer = 5;
    const ATTITUDE_INDICATOR_BORESIGHT_FOREGROUND: Layer = 6;
    const ATTITUDE_INDICATOR_VECTOR: Layer = 7;
}
