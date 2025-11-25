# 2D vs 3D CAD: Complete Technical Guide

> **対象**: CAD開発者、3Dグラフィックスエンジニア
> 
> **目的**: 2D/3D CADの違いと、3D自由視点カメラの全技術を網羅

---

## 📚 Table of Contents
1. [2D vs 3D: Fundamental Differences](#1-2d-vs-3d-fundamental-differences)
2. [3D Camera Systems](#2-3d-camera-systems)
3. [Projection Methods](#3-projection-methods)
4. [3D Viewport Controls](#4-3d-viewport-controls)
5. [Implementation Examples](#5-implementation-examples)

---

## 1. 2D vs 3D: Fundamental Differences

### 1.1 データ構造の違い

#### 2D CAD
```rust
struct Point2D {
    x: f32,
    y: f32,
}

struct Line2D {
    start: Point2D,
    end: Point2D,
}

struct Circle2D {
    center: Point2D,
    radius: f32,
}
```

#### 3D CAD
```rust
struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}

struct Line3D {
    start: Point3D,
    end: Point3D,
}

// 3Dでは「面」の概念が必要
struct Face {
    vertices: Vec<Point3D>,
    normal: Vector3, // 法線ベクトル
}

// B-rep (Boundary Representation)
struct Solid {
    faces: Vec<Face>,
    edges: Vec<Edge>,
    vertices: Vec<Vertex>,
}
```

---

### 1.2 座標変換の違い

#### 2D CAD: 3つの変換
```rust
// 1. Model Matrix (2D)
let model = Matrix3::from_translation(position)
    * Matrix3::from_rotation(angle)
    * Matrix3::from_scale(scale);

// 2. View Matrix (2D)
let view = Matrix3::from_translation(-camera_pos);

// 3. Projection Matrix (2D - Orthographic only)
let projection = Matrix3::from_scale(zoom);
```

#### 3D CAD: 4つの変換
```rust
// 1. Model Matrix (3D)
let model = Matrix4::from_translation(position)
    * Matrix4::from_rotation_quaternion(rotation)
    * Matrix4::from_scale(scale);

// 2. View Matrix (3D)
let view = Matrix4::look_at_rh(
    camera_position,
    target_position,
    up_vector
);

// 3. Projection Matrix (3D - Orthographic OR Perspective)
let projection = if orthographic {
    Matrix4::orthographic_rh(left, right, bottom, top, near, far)
} else {
    Matrix4::perspective_rh(fov, aspect_ratio, near, far)
};

// 4. Viewport Transform
let viewport = Matrix4::from_translation(Vector3::new(width/2, height/2, 0))
    * Matrix4::from_scale(Vector3::new(width/2, -height/2, 1));
```

---

### 1.3 レンダリングの違い

| 要素 | 2D CAD | 3D CAD |
|------|--------|--------|
| **深度バッファ** | 不要（Z順でソート） | 必須 |
| **法線ベクトル** | 不要 | 必須（ライティング） |
| **裏面カリング** | 不要 | 必須（性能向上） |
| **シェーディング** | Flat | Flat, Gouraud, Phong |
| **テクスチャ** | 稀 | 一般的 |

---

## 2. 3D Camera Systems

### 2.1 カメラの表現方法

#### Euler Angles (オイラー角) - ❌ 非推奨
```rust
struct EulerCamera {
    position: Vector3,
    pitch: f32,   // X軸回転（上下）
    yaw: f32,     // Y軸回転（左右）
    roll: f32,    // Z軸回転（傾き）
}

// 問題: Gimbal Lock（ジンバルロック）
// pitch が ±90度の時、yaw と roll が同じ軸になる
```

**Gimbal Lock の例**:
```
初期状態:
  Pitch = 0°, Yaw = 0°, Roll = 0°
  → 3軸すべて独立

Pitch = 90° にすると:
  Yaw軸とRoll軸が平行になる
  → 1自由度を失う（回転できない方向が発生）
```

#### Quaternion (クォータニオン) - ✅ 推奨
```rust
struct QuaternionCamera {
    position: Vector3,
    orientation: Quaternion, // (x, y, z, w)
}

impl QuaternionCamera {
    fn rotate(&mut self, axis: Vector3, angle: f32) {
        let rotation = Quaternion::from_axis_angle(axis, angle);
        self.orientation = rotation * self.orientation;
        self.orientation = self.orientation.normalize();
    }
    
    fn get_view_matrix(&self) -> Matrix4 {
        let rotation_matrix = Matrix4::from(self.orientation);
        let translation_matrix = Matrix4::from_translation(-self.position);
        rotation_matrix * translation_matrix
    }
}
```

**Quaternion のメリット**:
- ✅ Gimbal Lock が発生しない
- ✅ 補間が滑らか（SLERP）
- ✅ メモリ効率が良い（4要素 vs 9要素の回転行列）

---

### 2.2 Look-At Matrix

```rust
fn look_at_rh(eye: Vector3, target: Vector3, up: Vector3) -> Matrix4 {
    // カメラの向き（Z軸）
    let forward = (target - eye).normalize();
    
    // カメラの右方向（X軸）
    let right = forward.cross(&up).normalize();
    
    // カメラの上方向（Y軸）を再計算
    let up = right.cross(&forward);
    
    Matrix4::new(
        right.x,    right.y,    right.z,    -right.dot(&eye),
        up.x,       up.y,       up.z,       -up.dot(&eye),
        -forward.x, -forward.y, -forward.z, forward.dot(&eye),
        0.0,        0.0,        0.0,        1.0,
    )
}
```

---

## 3. Projection Methods

### 3.1 Orthographic Projection (正投影)

**特徴**: 平行投影、遠近感なし

```rust
fn orthographic_rh(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32
) -> Matrix4 {
    let width = right - left;
    let height = top - bottom;
    let depth = far - near;
    
    Matrix4::new(
        2.0 / width, 0.0,          0.0,         -(right + left) / width,
        0.0,         2.0 / height, 0.0,         -(top + bottom) / height,
        0.0,         0.0,          -2.0 / depth, -(far + near) / depth,
        0.0,         0.0,          0.0,         1.0,
    )
}
```

**用途**:
- ✅ 機械設計（寸法が正確）
- ✅ 建築図面（平面図、立面図）
- ✅ 2D CAD

**視覚的特徴**:
```
Orthographic:
  遠くの立方体 ■ = 近くの立方体 ■ （同じサイズ）
```

---

### 3.2 Perspective Projection (透視投影)

**特徴**: 遠近法、人間の視覚に近い

```rust
fn perspective_rh(
    fov_y: f32,      // 視野角（ラジアン）
    aspect: f32,     // アスペクト比 (width / height)
    near: f32,       // Near Plane
    far: f32         // Far Plane
) -> Matrix4 {
    let f = 1.0 / (fov_y / 2.0).tan();
    
    Matrix4::new(
        f / aspect, 0.0, 0.0,                        0.0,
        0.0,        f,   0.0,                        0.0,
        0.0,        0.0, (far + near) / (near - far), (2.0 * far * near) / (near - far),
        0.0,        0.0, -1.0,                       0.0,
    )
}
```

**用途**:
- ✅ プレゼンテーション
- ✅ レンダリング
- ✅ ゲーム

**視覚的特徴**:
```
Perspective:
  遠くの立方体 ▪ < 近くの立方体 ■ （遠いほど小さい）
```

---

### 3.3 CADでの使い分け

```rust
struct CadViewport {
    projection_mode: ProjectionMode,
}

enum ProjectionMode {
    Orthographic,
    Perspective,
    PerspectiveWithOrthoFaces, // AutoCAD風
}

impl CadViewport {
    fn get_projection_matrix(&self) -> Matrix4 {
        match self.projection_mode {
            ProjectionMode::Orthographic => {
                Matrix4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 1000.0)
            }
            ProjectionMode::Perspective => {
                Matrix4::perspective_rh(45.0_f32.to_radians(), 16.0/9.0, 0.1, 1000.0)
            }
            ProjectionMode::PerspectiveWithOrthoFaces => {
                // 視点が正面/側面/上面の時は Orthographic
                // それ以外は Perspective
                if self.is_aligned_to_axis() {
                    Matrix4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 1000.0)
                } else {
                    Matrix4::perspective_rh(45.0_f32.to_radians(), 16.0/9.0, 0.1, 1000.0)
                }
            }
        }
    }
}
```

---

## 4. 3D Viewport Controls

### 4.1 Orbit Controls (オービット)

**概念**: カメラが対象の周りを回転

```rust
struct OrbitCamera {
    target: Vector3,      // 注視点
    distance: f32,        // 距離
    azimuth: f32,         // 方位角（水平回転）
    elevation: f32,       // 仰角（垂直回転）
    up: Vector3,          // 上方向（通常は Y軸）
}

impl OrbitCamera {
    fn get_position(&self) -> Vector3 {
        let x = self.distance * self.elevation.cos() * self.azimuth.sin();
        let y = self.distance * self.elevation.sin();
        let z = self.distance * self.elevation.cos() * self.azimuth.cos();
        
        self.target + Vector3::new(x, y, z)
    }
    
    fn handle_mouse_drag(&mut self, delta_x: f32, delta_y: f32) {
        self.azimuth += delta_x * 0.01;
        self.elevation += delta_y * 0.01;
        
        // 仰角を制限（真上・真下を防ぐ）
        self.elevation = self.elevation.clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
    }
}
```

---

### 4.2 Arcball Controls (アークボール)

**概念**: 仮想球面上をドラッグ

```rust
struct ArcballCamera {
    center: Vector3,
    radius: f32,
    rotation: Quaternion,
}

impl ArcballCamera {
    fn screen_to_arcball(&self, screen_x: f32, screen_y: f32, screen_size: (f32, f32)) -> Vector3 {
        // スクリーン座標を [-1, 1] に正規化
        let x = (2.0 * screen_x / screen_size.0) - 1.0;
        let y = 1.0 - (2.0 * screen_y / screen_size.1);
        
        let length_squared = x * x + y * y;
        
        if length_squared <= 1.0 {
            // 球面上の点
            let z = (1.0 - length_squared).sqrt();
            Vector3::new(x, y, z).normalize()
        } else {
            // 球面外 → 円周上に投影
            Vector3::new(x, y, 0.0).normalize()
        }
    }
    
    fn handle_drag(&mut self, start: (f32, f32), end: (f32, f32), screen_size: (f32, f32)) {
        let v1 = self.screen_to_arcball(start.0, start.1, screen_size);
        let v2 = self.screen_to_arcball(end.0, end.1, screen_size);
        
        // 回転軸と角度を計算
        let axis = v1.cross(&v2);
        let angle = v1.dot(&v2).acos();
        
        // Quaternion で回転を適用
        let rotation = Quaternion::from_axis_angle(axis, angle);
        self.rotation = rotation * self.rotation;
    }
}
```

**Arcball の利点**:
- ✅ 直感的（球を回すような操作感）
- ✅ Gimbal Lock なし
- ✅ 滑らかな回転

---

### 4.3 Trackball Controls (トラックボール)

**Arcball との違い**:
- Arcball: 半球のみ（上下反転なし）
- Trackball: 全球（上下反転あり）

```rust
struct TrackballCamera {
    rotation: Quaternion,
    inertia: Vector3, // 慣性
}

impl TrackballCamera {
    fn handle_drag(&mut self, delta: Vector2, dt: f32) {
        let axis = Vector3::new(-delta.y, delta.x, 0.0).normalize();
        let angle = delta.length() * 0.01;
        
        let rotation = Quaternion::from_axis_angle(axis, angle);
        self.rotation = rotation * self.rotation;
        
        // 慣性を更新
        self.inertia = axis * angle / dt;
    }
    
    fn update(&mut self, dt: f32) {
        // 慣性で回転を継続
        if self.inertia.length() > 0.001 {
            let angle = self.inertia.length() * dt;
            let axis = self.inertia.normalize();
            
            let rotation = Quaternion::from_axis_angle(axis, angle);
            self.rotation = rotation * self.rotation;
            
            // 減衰
            self.inertia *= 0.95;
        }
    }
}
```

---

### 4.4 Pan Controls (パン)

```rust
impl Camera3D {
    fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let right = self.get_right_vector();
        let up = self.get_up_vector();
        
        let pan_speed = self.distance * 0.001; // 距離に応じて速度調整
        
        self.target += right * delta_x * pan_speed;
        self.target += up * delta_y * pan_speed;
    }
    
    fn get_right_vector(&self) -> Vector3 {
        let forward = (self.target - self.position).normalize();
        forward.cross(&self.up).normalize()
    }
    
    fn get_up_vector(&self) -> Vector3 {
        let forward = (self.target - self.position).normalize();
        let right = self.get_right_vector();
        right.cross(&forward)
    }
}
```

---

### 4.5 Zoom Controls (ズーム)

#### Orthographic Zoom
```rust
impl OrthographicCamera {
    fn zoom(&mut self, delta: f32) {
        self.zoom_level *= 1.0 + delta * 0.1;
        self.zoom_level = self.zoom_level.clamp(0.1, 100.0);
    }
}
```

#### Perspective Zoom (Dolly)
```rust
impl PerspectiveCamera {
    fn zoom(&mut self, delta: f32) {
        // 距離を変更（Dolly）
        self.distance *= 1.0 - delta * 0.1;
        self.distance = self.distance.clamp(1.0, 1000.0);
    }
    
    // または FOV を変更
    fn zoom_fov(&mut self, delta: f32) {
        self.fov += delta * 0.01;
        self.fov = self.fov.clamp(10.0_f32.to_radians(), 120.0_f32.to_radians());
    }
}
```

---

## 5. Implementation Examples

### 5.1 統合カメラシステム

```rust
struct UnifiedCamera {
    // 共通
    position: Vector3,
    target: Vector3,
    
    // 回転（Quaternion）
    orientation: Quaternion,
    
    // Projection
    projection_mode: ProjectionMode,
    ortho_zoom: f32,
    perspective_fov: f32,
    
    // Control Mode
    control_mode: ControlMode,
}

enum ControlMode {
    Orbit,
    Arcball,
    Trackball,
    FirstPerson,
}

impl UnifiedCamera {
    fn handle_input(&mut self, input: &Input) {
        match self.control_mode {
            ControlMode::Orbit => self.handle_orbit(input),
            ControlMode::Arcball => self.handle_arcball(input),
            ControlMode::Trackball => self.handle_trackball(input),
            ControlMode::FirstPerson => self.handle_first_person(input),
        }
    }
    
    fn get_view_projection_matrix(&self) -> Matrix4 {
        let view = self.get_view_matrix();
        let projection = self.get_projection_matrix();
        projection * view
    }
}
```

---

### 5.2 マウス入力処理

```rust
struct CameraController {
    camera: UnifiedCamera,
    last_mouse_pos: Option<(f32, f32)>,
    is_rotating: bool,
    is_panning: bool,
}

impl CameraController {
    fn handle_mouse_event(&mut self, event: &MouseEvent) {
        match event {
            MouseEvent::ButtonDown { button, pos } => {
                self.last_mouse_pos = Some(*pos);
                match button {
                    MouseButton::Left => self.is_rotating = true,
                    MouseButton::Middle => self.is_panning = true,
                    _ => {}
                }
            }
            
            MouseEvent::ButtonUp { .. } => {
                self.is_rotating = false;
                self.is_panning = false;
                self.last_mouse_pos = None;
            }
            
            MouseEvent::Move { pos } => {
                if let Some(last_pos) = self.last_mouse_pos {
                    let delta = (pos.0 - last_pos.0, pos.1 - last_pos.1);
                    
                    if self.is_rotating {
                        self.camera.rotate(delta);
                    } else if self.is_panning {
                        self.camera.pan(delta.0, delta.1);
                    }
                    
                    self.last_mouse_pos = Some(*pos);
                }
            }
            
            MouseEvent::Wheel { delta } => {
                self.camera.zoom(*delta);
            }
        }
    }
}
```

---

### 5.3 標準ビュー（三面図）

```rust
impl UnifiedCamera {
    fn set_standard_view(&mut self, view: StandardView) {
        match view {
            StandardView::Top => {
                self.position = self.target + Vector3::new(0.0, 100.0, 0.0);
                self.orientation = Quaternion::look_rotation(
                    Vector3::new(0.0, -1.0, 0.0),
                    Vector3::new(0.0, 0.0, -1.0)
                );
            }
            StandardView::Front => {
                self.position = self.target + Vector3::new(0.0, 0.0, 100.0);
                self.orientation = Quaternion::look_rotation(
                    Vector3::new(0.0, 0.0, -1.0),
                    Vector3::new(0.0, 1.0, 0.0)
                );
            }
            StandardView::Right => {
                self.position = self.target + Vector3::new(100.0, 0.0, 0.0);
                self.orientation = Quaternion::look_rotation(
                    Vector3::new(-1.0, 0.0, 0.0),
                    Vector3::new(0.0, 1.0, 0.0)
                );
            }
            StandardView::Isometric => {
                let angle = 35.264_f32.to_radians(); // arctan(1/sqrt(2))
                self.position = self.target + Vector3::new(100.0, 100.0, 100.0);
                // ...
            }
        }
        
        // 標準ビューは Orthographic
        self.projection_mode = ProjectionMode::Orthographic;
    }
}

enum StandardView {
    Top,
    Bottom,
    Front,
    Back,
    Left,
    Right,
    Isometric,
}
```

---

## 📊 比較表

### 2D vs 3D CAD

| 要素 | 2D CAD | 3D CAD |
|------|--------|--------|
| **座標** | (x, y) | (x, y, z) |
| **回転** | 1軸（Z軸のみ） | 3軸（Quaternion推奨） |
| **投影** | Orthographic のみ | Ortho / Perspective |
| **カメラ制御** | Pan, Zoom | Orbit, Pan, Zoom |
| **深度** | Z-order | Depth Buffer |
| **法線** | 不要 | 必須 |
| **複雑度** | ⭐⭐ | ⭐⭐⭐⭐⭐ |

### カメラ制御方式

| 方式 | 直感性 | Gimbal Lock | 慣性 | 用途 |
|------|--------|-------------|------|------|
| Orbit | ⭐⭐⭐ | ❌ | ❌ | CAD標準 |
| Arcball | ⭐⭐⭐⭐ | ✅ | ❌ | 直感的操作 |
| Trackball | ⭐⭐⭐⭐ | ✅ | ✅ | 自由回転 |
| First Person | ⭐⭐ | ❌ | ❌ | ゲーム |

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-25*
