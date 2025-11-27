use cgmath::*;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub zoom: f32,
    pub pan: Vector2<f32>,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            eye: (0.0, 0.0, 1.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vector3::unit_y(),
            aspect: width / height,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            zoom: 1.0,
            pan: Vector2::new(0.0, 0.0),
        }
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        // For 2D CAD, we usually use Orthographic projection, but let's stick to simple 2D transform first.
        // Or use perspective if we want 3D later.
        // Let's implement a simple 2D view matrix: Scale (zoom) -> Translate (pan)
        
        let view = Matrix4::from_translation(Vector3::new(self.pan.x, self.pan.y, 0.0)) * 
                   Matrix4::from_scale(self.zoom);
        
        // Correction for aspect ratio to keep square things square
        let aspect_correction = if self.aspect > 1.0 {
             Matrix4::from_nonuniform_scale(1.0 / self.aspect, 1.0, 1.0)
        } else {
             Matrix4::from_nonuniform_scale(1.0, self.aspect, 1.0)
        };

        aspect_correction * view
    }
    
    pub fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }

    pub fn screen_to_world(&self, screen_pos: Vector2<f32>, screen_width: f32, screen_height: f32) -> Vector2<f32> {
        // Convert screen (0..width, 0..height) to NDC (-1..1, 1..-1)
        let x = (screen_pos.x / screen_width) * 2.0 - 1.0;
        let y = -((screen_pos.y / screen_height) * 2.0 - 1.0); // Flip Y for NDC
        
        let ndc = Vector4::new(x, y, 0.0, 1.0);
        
        // Inverse view projection
        let view_proj = self.build_view_projection_matrix();
        let inv_view_proj = view_proj.invert().unwrap_or(Matrix4::identity());
        
        let world = inv_view_proj * ndc;
        Vector2::new(world.x, world.y)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
