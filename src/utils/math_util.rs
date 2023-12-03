pub fn axis_angle_to_quat(angle_radian: f32, axis: glam::Vec3) -> glam::Quat {
    let s = f32::sin(angle_radian * 0.5);
    glam::quat(axis.x * s, axis.y * s, axis.z * s, f32::cos(angle_radian))
}
