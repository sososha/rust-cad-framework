# Complete CAD Drawing Commands Reference

> **対象**: CAD開発者、全作図コマンドの実装を目指す開発者
> 
> **目的**: JWW CAD, AutoCAD, Vectorworks, FreeCAD の全コマンドを網羅し、実装方法を解説

---

## 📚 Table of Contents
1. [Command Categories](#1-command-categories)
2. [Basic Drawing Commands](#2-basic-drawing-commands)
3. [Advanced Drawing Commands](#3-advanced-drawing-commands)
4. [Modification Commands](#4-modification-commands)
5. [Constraint & Parametric Commands](#5-constraint--parametric-commands)
6. [Implementation Patterns](#6-implementation-patterns)

---

## 1. Command Categories

### 1.1 コマンド分類

| カテゴリ | 説明 | 例 |
|---------|------|-----|
| **Basic Drawing** | 基本図形作成 | Line, Circle, Rectangle |
| **Advanced Drawing** | 高度な図形 | Spline, Polygon, Hatch |
| **Modification** | 編集・変形 | Move, Copy, Trim, Extend |
| **Constraint** | 拘束 | Parallel, Perpendicular, Tangent |
| **Dimension** | 寸法 | Linear, Angular, Radial |
| **Utility** | 補助機能 | Snap, Layer, Block |

---

## 2. Basic Drawing Commands

### 2.1 Line (線)

**JWW CAD**: 線  
**AutoCAD**: LINE (L)  
**Vectorworks**: Line Tool (2)  
**FreeCAD**: Sketcher Line

#### マウスイベント
```rust
struct LineTool {
    start: Option<Point>,
    preview: Option<Point>,
}

impl Tool for LineTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        if self.start.is_none() {
            // 始点を記録
            self.start = Some(pos);
        } else {
            // 終点で確定
            let entity = Entity::Line {
                p1: self.start.unwrap(),
                p2: pos,
            };
            state.geometry.add_entity(entity);
            self.start = None;
        }
    }
    
    fn mouse_move(&mut self, pos: Point, state: &mut AppState) {
        // プレビュー更新
        if self.start.is_some() {
            self.preview = Some(pos);
        }
    }
    
    fn render_preview(&self, renderer: &mut Renderer) {
        if let (Some(start), Some(end)) = (self.start, self.preview) {
            renderer.draw_line_dashed(start, end, Color::GRAY);
        }
    }
}
```

---

### 2.2 Circle (円)

**JWW CAD**: 円弧  
**AutoCAD**: CIRCLE (C)  
**Vectorworks**: Circle Tool (Alt+6)  
**FreeCAD**: Sketcher Circle

#### 入力方式
1. **Center + Radius** (中心 + 半径)
2. **Center + Diameter** (中心 + 直径)
3. **3 Points** (3点指定)
4. **2 Points** (2点指定 - 直径)
5. **Tangent + Radius** (接線 + 半径)

```rust
enum CircleMode {
    CenterRadius,
    CenterDiameter,
    ThreePoints,
    TwoPoints,
    TangentRadius,
}

struct CircleTool {
    mode: CircleMode,
    center: Option<Point>,
    radius: Option<f32>,
    points: Vec<Point>,
}

impl Tool for CircleTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        match self.mode {
            CircleMode::CenterRadius => {
                if self.center.is_none() {
                    self.center = Some(pos);
                } else {
                    let radius = (pos - self.center.unwrap()).len();
                    let entity = Entity::Circle {
                        center: self.center.unwrap(),
                        radius,
                    };
                    state.geometry.add_entity(entity);
                    self.center = None;
                }
            }
            CircleMode::ThreePoints => {
                self.points.push(pos);
                if self.points.len() == 3 {
                    let circle = self.circle_from_three_points(&self.points);
                    state.geometry.add_entity(Entity::Circle {
                        center: circle.center,
                        radius: circle.radius,
                    });
                    self.points.clear();
                }
            }
            _ => {}
        }
    }
    
    fn circle_from_three_points(&self, points: &[Point]) -> Circle {
        // 3点から円を計算
        let (p1, p2, p3) = (points[0], points[1], points[2]);
        
        // 外接円の中心を計算
        let d = 2.0 * (p1.x * (p2.y - p3.y) + p2.x * (p3.y - p1.y) + p3.x * (p1.y - p2.y));
        
        let ux = ((p1.x * p1.x + p1.y * p1.y) * (p2.y - p3.y) +
                  (p2.x * p2.x + p2.y * p2.y) * (p3.y - p1.y) +
                  (p3.x * p3.x + p3.y * p3.y) * (p1.y - p2.y)) / d;
        
        let uy = ((p1.x * p1.x + p1.y * p1.y) * (p3.x - p2.x) +
                  (p2.x * p2.x + p2.y * p2.y) * (p1.x - p3.x) +
                  (p3.x * p3.x + p3.y * p3.y) * (p2.x - p1.x)) / d;
        
        let center = Point::new(ux, uy);
        let radius = (p1 - center).len();
        
        Circle { center, radius }
    }
}
```

---

### 2.3 Rectangle (矩形)

**JWW CAD**: 矩形  
**AutoCAD**: RECTANGLE (REC)  
**Vectorworks**: Rectangle Tool (4)  
**FreeCAD**: Sketcher Rectangle

#### 入力方式
1. **Corner to Corner** (対角2点)
2. **Center + Corner** (中心 + 角)
3. **3 Points** (3点指定)

```rust
enum RectangleMode {
    CornerToCorner,
    CenterCorner,
    ThreePoints,
}

struct RectangleTool {
    mode: RectangleMode,
    first_point: Option<Point>,
}

impl Tool for RectangleTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        match self.mode {
            RectangleMode::CornerToCorner => {
                if self.first_point.is_none() {
                    self.first_point = Some(pos);
                } else {
                    let p1 = self.first_point.unwrap();
                    let p2 = pos;
                    
                    // 4つの頂点を計算
                    let vertices = vec![
                        p1,
                        Point::new(p2.x, p1.y),
                        p2,
                        Point::new(p1.x, p2.y),
                    ];
                    
                    state.geometry.add_entity(Entity::Polyline { vertices });
                    self.first_point = None;
                }
            }
            _ => {}
        }
    }
}
```

---

### 2.4 Arc (円弧)

**JWW CAD**: 円弧  
**AutoCAD**: ARC (A)  
**Vectorworks**: Arc Tool  
**FreeCAD**: Sketcher Arc

#### 入力方式
1. **3 Points** (3点指定)
2. **Start + Center + End** (始点 + 中心 + 終点)
3. **Start + End + Radius** (始点 + 終点 + 半径)

```rust
enum ArcMode {
    ThreePoints,
    StartCenterEnd,
    StartEndRadius,
}

struct ArcTool {
    mode: ArcMode,
    points: Vec<Point>,
}

impl Tool for ArcTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        self.points.push(pos);
        
        match self.mode {
            ArcMode::ThreePoints if self.points.len() == 3 => {
                let arc = self.arc_from_three_points(&self.points);
                state.geometry.add_entity(Entity::Arc(arc));
                self.points.clear();
            }
            ArcMode::StartCenterEnd if self.points.len() == 3 => {
                let (start, center, end) = (self.points[0], self.points[1], self.points[2]);
                let arc = Arc {
                    center,
                    radius: (start - center).len(),
                    start_angle: (start - center).angle(),
                    end_angle: (end - center).angle(),
                };
                state.geometry.add_entity(Entity::Arc(arc));
                self.points.clear();
            }
            _ => {}
        }
    }
}
```

---

### 2.5 Polyline (連続線)

**JWW CAD**: 連続線  
**AutoCAD**: POLYLINE (PL)  
**Vectorworks**: Polyline Tool (5)  
**FreeCAD**: Sketcher Polyline

```rust
struct PolylineTool {
    vertices: Vec<Point>,
    closed: bool,
}

impl Tool for PolylineTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        self.vertices.push(pos);
    }
    
    fn key_down(&mut self, key: Key, state: &mut AppState) {
        match key {
            Key::Enter | Key::Escape => {
                // 確定
                if self.vertices.len() >= 2 {
                    state.geometry.add_entity(Entity::Polyline {
                        vertices: self.vertices.clone(),
                    });
                }
                self.vertices.clear();
            }
            Key::C => {
                // Close (閉じる)
                self.closed = true;
                if self.vertices.len() >= 3 {
                    let mut verts = self.vertices.clone();
                    verts.push(verts[0]); // 始点に戻る
                    state.geometry.add_entity(Entity::Polyline { vertices: verts });
                }
                self.vertices.clear();
            }
            _ => {}
        }
    }
}
```

---

## 3. Advanced Drawing Commands

### 3.1 Polygon (多角形)

**JWW CAD**: 多角形  
**AutoCAD**: POLYGON (POL)  
**Vectorworks**: 2D Polygon Tool  
**FreeCAD**: (Polyline + Constraints)

```rust
struct PolygonTool {
    center: Option<Point>,
    num_sides: usize,
    radius: Option<f32>,
    inscribed: bool, // true: 内接, false: 外接
}

impl Tool for PolygonTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        if self.center.is_none() {
            self.center = Some(pos);
        } else {
            let radius = (pos - self.center.unwrap()).len();
            let vertices = self.calculate_polygon_vertices(
                self.center.unwrap(),
                radius,
                self.num_sides,
                self.inscribed
            );
            
            state.geometry.add_entity(Entity::Polyline { vertices });
            self.center = None;
        }
    }
    
    fn calculate_polygon_vertices(
        &self,
        center: Point,
        radius: f32,
        sides: usize,
        inscribed: bool
    ) -> Vec<Point> {
        let mut vertices = Vec::new();
        let angle_step = 2.0 * std::f32::consts::PI / sides as f32;
        
        let actual_radius = if inscribed {
            radius
        } else {
            radius / (angle_step / 2.0).cos()
        };
        
        for i in 0..sides {
            let angle = i as f32 * angle_step;
            let x = center.x + actual_radius * angle.cos();
            let y = center.y + actual_radius * angle.sin();
            vertices.push(Point::new(x, y));
        }
        
        // 閉じる
        vertices.push(vertices[0]);
        vertices
    }
}
```

---

### 3.2 Spline (スプライン曲線)

**AutoCAD**: SPLINE (SPL)  
**Vectorworks**: Polyline (Bezier mode)  
**FreeCAD**: Sketcher B-Spline

```rust
struct SplineTool {
    control_points: Vec<Point>,
    degree: usize, // 次数 (通常3)
}

impl Tool for SplineTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        self.control_points.push(pos);
    }
    
    fn key_down(&mut self, key: Key, state: &mut AppState) {
        if key == Key::Enter && self.control_points.len() >= 2 {
            let spline = self.create_spline();
            state.geometry.add_entity(Entity::Spline(spline));
            self.control_points.clear();
        }
    }
    
    fn create_spline(&self) -> Spline {
        // B-Spline 曲線を生成
        Spline {
            control_points: self.control_points.clone(),
            degree: self.degree,
            knots: self.calculate_knots(),
        }
    }
    
    fn calculate_knots(&self) -> Vec<f32> {
        // Uniform knot vector
        let n = self.control_points.len();
        let m = n + self.degree + 1;
        
        (0..m).map(|i| i as f32 / (m - 1) as f32).collect()
    }
}
```

---

### 3.3 Hatch (ハッチング)

**JWW CAD**: ハッチ  
**AutoCAD**: HATCH (H)  
**Vectorworks**: Hatch Tool  
**FreeCAD**: (Draft Hatch)

```rust
enum HatchPattern {
    Solid,
    Lines { angle: f32, spacing: f32 },
    CrossHatch { angle1: f32, angle2: f32, spacing: f32 },
    Dots { spacing: f32 },
}

struct HatchTool {
    pattern: HatchPattern,
    boundary: Vec<Point>,
}

impl Tool for HatchTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        // 境界を選択
        if let Some(entity_id) = state.geometry.find_at_point(pos, 5.0) {
            if let Some(entity) = state.geometry.get_entity(entity_id) {
                self.boundary = entity.get_boundary_points();
                
                // ハッチングを生成
                let hatch = self.generate_hatch();
                state.geometry.add_entity(Entity::Hatch {
                    boundary: self.boundary.clone(),
                    pattern: self.pattern.clone(),
                    lines: hatch,
                });
            }
        }
    }
    
    fn generate_hatch(&self) -> Vec<Line> {
        match &self.pattern {
            HatchPattern::Lines { angle, spacing } => {
                self.generate_line_hatch(*angle, *spacing)
            }
            HatchPattern::Solid => vec![],
            _ => vec![],
        }
    }
    
    fn generate_line_hatch(&self, angle: f32, spacing: f32) -> Vec<Line> {
        // 境界内に線を生成
        let mut lines = Vec::new();
        
        // 境界のバウンディングボックスを計算
        let bbox = self.calculate_bounding_box(&self.boundary);
        
        // 角度方向に線を生成
        let direction = Vector2::new(angle.cos(), angle.sin());
        let perpendicular = Vector2::new(-angle.sin(), angle.cos());
        
        let mut offset = 0.0;
        while offset < bbox.width {
            let start = bbox.min + perpendicular * offset;
            let end = start + direction * bbox.height;
            
            // 境界との交点を計算
            if let Some(clipped_line) = self.clip_line_to_boundary(start, end) {
                lines.push(clipped_line);
            }
            
            offset += spacing;
        }
        
        lines
    }
}
```

---

## 4. Modification Commands

### 4.1 Move (移動)

**JWW CAD**: 図形移動  
**AutoCAD**: MOVE (M)  
**Vectorworks**: Move (Ctrl+M)  
**FreeCAD**: (Sketcher Move)

```rust
struct MoveTool {
    selected_entities: Vec<EntityId>,
    base_point: Option<Point>,
    offset: Vector2,
}

impl Tool for MoveTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        if self.base_point.is_none() {
            // 基準点を設定
            self.base_point = Some(pos);
        } else {
            // 移動を確定
            self.offset = pos - self.base_point.unwrap();
            
            for id in &self.selected_entities {
                if let Some(entity) = state.geometry.get_entity_mut(*id) {
                    entity.translate(self.offset);
                }
            }
            
            self.base_point = None;
            self.selected_entities.clear();
        }
    }
}
```

---

### 4.2 Copy (複写)

**JWW CAD**: コピー, 図形複写  
**AutoCAD**: COPY (CO, CP)  
**Vectorworks**: Copy  
**FreeCAD**: Clone

```rust
struct CopyTool {
    selected_entities: Vec<EntityId>,
    base_point: Option<Point>,
    multiple: bool, // 連続複写
}

impl Tool for CopyTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        if self.base_point.is_none() {
            self.base_point = Some(pos);
        } else {
            let offset = pos - self.base_point.unwrap();
            
            for id in &self.selected_entities {
                if let Some(entity) = state.geometry.get_entity(*id) {
                    let mut new_entity = entity.clone();
                    new_entity.translate(offset);
                    state.geometry.add_entity(new_entity);
                }
            }
            
            if !self.multiple {
                self.base_point = None;
                self.selected_entities.clear();
            }
        }
    }
}
```

---

### 4.3 Offset (複線/オフセット)

**JWW CAD**: 複線  
**AutoCAD**: OFFSET (O)  
**Vectorworks**: Offset (Shift+-)  
**FreeCAD**: (Sketcher Offset)

```rust
struct OffsetTool {
    distance: f32,
    side: OffsetSide,
}

enum OffsetSide {
    Left,
    Right,
    Both,
}

impl Tool for OffsetTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        // クリック位置に最も近いエンティティを取得
        if let Some(entity_id) = state.geometry.find_nearest(pos, 10.0) {
            if let Some(entity) = state.geometry.get_entity(entity_id) {
                match entity {
                    Entity::Line { p1, p2 } => {
                        let offset_lines = self.offset_line(*p1, *p2, pos);
                        for line in offset_lines {
                            state.geometry.add_entity(Entity::Line {
                                p1: line.0,
                                p2: line.1,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    fn offset_line(&self, p1: Point, p2: Point, click_pos: Point) -> Vec<(Point, Point)> {
        // 線の方向ベクトル
        let direction = (p2 - p1).normalize();
        
        // 垂直ベクトル
        let perpendicular = Vector2::new(-direction.y, direction.x);
        
        // クリック位置がどちら側か判定
        let to_click = click_pos - p1;
        let side_sign = if to_click.dot(&perpendicular) > 0.0 { 1.0 } else { -1.0 };
        
        let offset = perpendicular * self.distance * side_sign;
        
        match self.side {
            OffsetSide::Left | OffsetSide::Right => {
                vec![(p1 + offset, p2 + offset)]
            }
            OffsetSide::Both => {
                vec![
                    (p1 + offset, p2 + offset),
                    (p1 - offset, p2 - offset),
                ]
            }
        }
    }
}
```

---

### 4.4 Trim (トリム)

**JWW CAD**: 伸縮  
**AutoCAD**: TRIM (TR)  
**Vectorworks**: Trim (Alt+Shift+L)  
**FreeCAD**: (Sketcher Trim)

```rust
struct TrimTool {
    cutting_edges: Vec<EntityId>,
}

impl Tool for TrimTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        // クリック位置のエンティティを取得
        if let Some(entity_id) = state.geometry.find_at_point(pos, 5.0) {
            if let Some(entity) = state.geometry.get_entity(entity_id) {
                // 切断エッジとの交点を計算
                let intersections = self.find_intersections(entity, &state.geometry);
                
                // クリック位置に最も近い2つの交点を見つける
                if let Some((trim_start, trim_end)) = self.find_trim_points(pos, &intersections) {
                    // エンティティをトリム
                    let trimmed = entity.trim(trim_start, trim_end);
                    state.geometry.remove_entity(entity_id);
                    state.geometry.add_entity(trimmed);
                }
            }
        }
    }
}
```

---

### 4.5 Extend (延長)

**JWW CAD**: 伸縮  
**AutoCAD**: EXTEND (EX)  
**Vectorworks**: (Trim の逆)  
**FreeCAD**: (Sketcher Extend)

```rust
struct ExtendTool {
    boundary_edges: Vec<EntityId>,
}

impl Tool for ExtendTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        if let Some(entity_id) = state.geometry.find_at_point(pos, 5.0) {
            if let Some(entity) = state.geometry.get_entity(entity_id) {
                // 境界エッジとの交点を計算
                if let Some(intersection) = self.find_nearest_intersection(entity, &state.geometry) {
                    // エンティティを延長
                    let extended = entity.extend_to(intersection);
                    state.geometry.remove_entity(entity_id);
                    state.geometry.add_entity(extended);
                }
            }
        }
    }
}
```

---

### 4.6 Fillet (フィレット/面取り)

**JWW CAD**: 面取  
**AutoCAD**: FILLET (F)  
**Vectorworks**: Fillet (7)  
**FreeCAD**: Sketcher Fillet

```rust
struct FilletTool {
    radius: f32,
    first_entity: Option<EntityId>,
}

impl Tool for FilletTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        if let Some(entity_id) = state.geometry.find_at_point(pos, 5.0) {
            if self.first_entity.is_none() {
                self.first_entity = Some(entity_id);
            } else {
                // 2つのエンティティ間にフィレットを作成
                let e1 = state.geometry.get_entity(self.first_entity.unwrap()).unwrap();
                let e2 = state.geometry.get_entity(entity_id).unwrap();
                
                if let Some(fillet) = self.create_fillet(e1, e2) {
                    state.geometry.add_entity(fillet);
                }
                
                self.first_entity = None;
            }
        }
    }
    
    fn create_fillet(&self, e1: &Entity, e2: &Entity) -> Option<Entity> {
        // 2つの線の交点を計算
        let intersection = e1.intersect(e2)?;
        
        // フィレット円弧の中心を計算
        let (dir1, dir2) = (e1.direction_at(intersection), e2.direction_at(intersection));
        let bisector = (dir1 + dir2).normalize();
        
        let angle = dir1.angle_to(&dir2);
        let distance = self.radius / (angle / 2.0).sin();
        
        let center = intersection + bisector * distance;
        
        // 接点を計算
        let tangent1 = center + dir1.perpendicular() * self.radius;
        let tangent2 = center + dir2.perpendicular() * self.radius;
        
        Some(Entity::Arc(Arc {
            center,
            radius: self.radius,
            start_angle: (tangent1 - center).angle(),
            end_angle: (tangent2 - center).angle(),
        }))
    }
}
```

---

### 4.7 Mirror (鏡像)

**JWW CAD**: (複写 + 反転)  
**AutoCAD**: MIRROR (MI)  
**Vectorworks**: Mirror (=)  
**FreeCAD**: Sketcher Mirror

```rust
struct MirrorTool {
    selected_entities: Vec<EntityId>,
    mirror_line_start: Option<Point>,
    mirror_line_end: Option<Point>,
    keep_original: bool,
}

impl Tool for MirrorTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        if self.mirror_line_start.is_none() {
            self.mirror_line_start = Some(pos);
        } else if self.mirror_line_end.is_none() {
            self.mirror_line_end = Some(pos);
            
            // 鏡像を作成
            let p1 = self.mirror_line_start.unwrap();
            let p2 = self.mirror_line_end.unwrap();
            
            for id in &self.selected_entities {
                if let Some(entity) = state.geometry.get_entity(*id) {
                    let mirrored = entity.mirror(p1, p2);
                    state.geometry.add_entity(mirrored);
                    
                    if !self.keep_original {
                        state.geometry.remove_entity(*id);
                    }
                }
            }
            
            self.mirror_line_start = None;
            self.mirror_line_end = None;
            self.selected_entities.clear();
        }
    }
}

impl Entity {
    fn mirror(&self, line_p1: Point, line_p2: Point) -> Entity {
        match self {
            Entity::Line { p1, p2 } => {
                Entity::Line {
                    p1: mirror_point(*p1, line_p1, line_p2),
                    p2: mirror_point(*p2, line_p1, line_p2),
                }
            }
            Entity::Circle { center, radius } => {
                Entity::Circle {
                    center: mirror_point(*center, line_p1, line_p2),
                    radius: *radius,
                }
            }
            _ => self.clone(),
        }
    }
}

fn mirror_point(point: Point, line_p1: Point, line_p2: Point) -> Point {
    let line_vec = (line_p2 - line_p1).normalize();
    let to_point = point - line_p1;
    
    // 線への投影
    let projection = line_vec * to_point.dot(&line_vec);
    
    // 鏡像点
    line_p1 + projection * 2.0 - to_point
}
```

---

### 4.8 Rotate (回転)

**JWW CAD**: (図形移動 + 回転)  
**AutoCAD**: ROTATE (RO)  
**Vectorworks**: Rotate (Alt+=)  
**FreeCAD**: (Sketcher Rotate)

```rust
struct RotateTool {
    selected_entities: Vec<EntityId>,
    center: Option<Point>,
    start_angle: Option<f32>,
}

impl Tool for RotateTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        if self.center.is_none() {
            self.center = Some(pos);
        } else if self.start_angle.is_none() {
            let vec = pos - self.center.unwrap();
            self.start_angle = Some(vec.angle());
        } else {
            // 回転角度を計算
            let vec = pos - self.center.unwrap();
            let end_angle = vec.angle();
            let rotation = end_angle - self.start_angle.unwrap();
            
            // エンティティを回転
            for id in &self.selected_entities {
                if let Some(entity) = state.geometry.get_entity_mut(*id) {
                    entity.rotate(self.center.unwrap(), rotation);
                }
            }
            
            self.center = None;
            self.start_angle = None;
            self.selected_entities.clear();
        }
    }
}
```

---

### 4.9 Scale (拡大縮小)

**AutoCAD**: SCALE (SC)  
**Vectorworks**: (Resize)  
**FreeCAD**: (Sketcher Scale)

```rust
struct ScaleTool {
    selected_entities: Vec<EntityId>,
    base_point: Option<Point>,
    scale_factor: f32,
}

impl Tool for ScaleTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        if self.base_point.is_none() {
            self.base_point = Some(pos);
        } else {
            // スケール係数を計算
            let distance = (pos - self.base_point.unwrap()).len();
            self.scale_factor = distance / 100.0; // 基準距離100
            
            for id in &self.selected_entities {
                if let Some(entity) = state.geometry.get_entity_mut(*id) {
                    entity.scale(self.base_point.unwrap(), self.scale_factor);
                }
            }
            
            self.base_point = None;
            self.selected_entities.clear();
        }
    }
}
```

---

### 4.10 Array (配列複写)

**JWW CAD**: (図形複写 + 連続)  
**AutoCAD**: ARRAY (AR)  
**Vectorworks**: (Array)  
**FreeCAD**: Rectangular Array

#### 配列タイプ
1. **Rectangular** (矩形配列)
2. **Polar** (円形配列)
3. **Path** (パス配列)

```rust
enum ArrayType {
    Rectangular { rows: usize, cols: usize, row_spacing: f32, col_spacing: f32 },
    Polar { center: Point, count: usize, angle: f32 },
    Path { path: Vec<Point>, count: usize },
}

struct ArrayTool {
    selected_entities: Vec<EntityId>,
    array_type: ArrayType,
}

impl Tool for ArrayTool {
    fn execute(&mut self, state: &mut AppState) {
        match &self.array_type {
            ArrayType::Rectangular { rows, cols, row_spacing, col_spacing } => {
                for row in 0..*rows {
                    for col in 0..*cols {
                        if row == 0 && col == 0 { continue; } // 元のエンティティはスキップ
                        
                        let offset = Vector2::new(
                            col as f32 * col_spacing,
                            row as f32 * row_spacing
                        );
                        
                        for id in &self.selected_entities {
                            if let Some(entity) = state.geometry.get_entity(*id) {
                                let mut new_entity = entity.clone();
                                new_entity.translate(offset);
                                state.geometry.add_entity(new_entity);
                            }
                        }
                    }
                }
            }
            ArrayType::Polar { center, count, angle } => {
                let angle_step = angle / (*count as f32);
                
                for i in 1..*count {
                    let rotation = angle_step * i as f32;
                    
                    for id in &self.selected_entities {
                        if let Some(entity) = state.geometry.get_entity(*id) {
                            let mut new_entity = entity.clone();
                            new_entity.rotate(*center, rotation);
                            state.geometry.add_entity(new_entity);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
```

---

## 5. Constraint & Parametric Commands

### 5.1 Parallel (平行)

**FreeCAD**: Sketcher Parallel  
**AutoCAD**: (Constraint)

```rust
struct ParallelConstraint {
    line1: EntityId,
    line2: EntityId,
}

impl Constraint for ParallelConstraint {
    fn apply(&self, state: &mut AppState) {
        let l1 = state.geometry.get_entity(self.line1).unwrap();
        let l2 = state.geometry.get_entity_mut(self.line2).unwrap();
        
        if let (Entity::Line { p1: p1_1, p2: p2_1 }, Entity::Line { p1: p1_2, p2: p2_2 }) = (l1, l2) {
            let dir1 = (*p2_1 - *p1_1).normalize();
            let length = (*p2_2 - *p1_2).len();
            
            // line2 を line1 と平行にする
            *p2_2 = *p1_2 + dir1 * length;
        }
    }
}
```

---

### 5.2 Perpendicular (垂直)

**FreeCAD**: Sketcher Perpendicular

```rust
struct PerpendicularConstraint {
    line1: EntityId,
    line2: EntityId,
}

impl Constraint for PerpendicularConstraint {
    fn apply(&self, state: &mut AppState) {
        let l1 = state.geometry.get_entity(self.line1).unwrap();
        let l2 = state.geometry.get_entity_mut(self.line2).unwrap();
        
        if let (Entity::Line { p1: p1_1, p2: p2_1 }, Entity::Line { p1: p1_2, p2: p2_2 }) = (l1, l2) {
            let dir1 = (*p2_1 - *p1_1).normalize();
            let perpendicular = Vector2::new(-dir1.y, dir1.x);
            let length = (*p2_2 - *p1_2).len();
            
            *p2_2 = *p1_2 + perpendicular * length;
        }
    }
}
```

---

### 5.3 Tangent (接線)

**JWW CAD**: 接線  
**FreeCAD**: Sketcher Tangent

```rust
struct TangentConstraint {
    line: EntityId,
    circle: EntityId,
}

impl Constraint for TangentConstraint {
    fn apply(&self, state: &mut AppState) {
        let line = state.geometry.get_entity_mut(self.line).unwrap();
        let circle = state.geometry.get_entity(self.circle).unwrap();
        
        if let (Entity::Line { p1, p2 }, Entity::Circle { center, radius }) = (line, circle) {
            // 線を円に接するように調整
            let dir = (*p2 - *p1).normalize();
            let to_center = *center - *p1;
            
            // 円の中心から線への距離が半径になるように調整
            let distance = to_center.dot(&dir.perpendicular()).abs();
            let adjustment = (*radius - distance) * dir.perpendicular().normalize();
            
            *p1 = *p1 + adjustment;
            *p2 = *p2 + adjustment;
        }
    }
}
```

---

## 6. Implementation Patterns

### 6.1 Tool State Machine

```rust
enum ToolState {
    Idle,
    WaitingForFirstPoint,
    WaitingForSecondPoint,
    WaitingForThirdPoint,
    Dragging,
    Previewing,
}

trait Tool {
    fn get_state(&self) -> ToolState;
    fn mouse_down(&mut self, pos: Point, state: &mut AppState);
    fn mouse_move(&mut self, pos: Point, state: &mut AppState);
    fn mouse_up(&mut self, pos: Point, state: &mut AppState);
    fn key_down(&mut self, key: Key, state: &mut AppState);
    fn render_preview(&self, renderer: &mut Renderer);
    fn cancel(&mut self);
}
```

---

### 6.2 Command Pattern for Undo/Redo

```rust
trait Command {
    fn execute(&mut self, state: &mut AppState);
    fn undo(&mut self, state: &mut AppState);
}

struct DrawLineCommand {
    line: Entity,
    entity_id: Option<EntityId>,
}

impl Command for DrawLineCommand {
    fn execute(&mut self, state: &mut AppState) {
        self.entity_id = Some(state.geometry.add_entity(self.line.clone()));
    }
    
    fn undo(&mut self, state: &mut AppState) {
        if let Some(id) = self.entity_id {
            state.geometry.remove_entity(id);
        }
    }
}
```

---

### 6.3 Snap System Integration

```rust
impl Tool for LineTool {
    fn mouse_move(&mut self, pos: Point, state: &mut AppState) {
        // スナップを適用
        let snapped_pos = if state.snap_enabled {
            state.snap_system.snap(pos, &state.geometry)
        } else {
            pos
        };
        
        self.preview = Some(snapped_pos);
    }
}

impl SnapSystem {
    fn snap(&self, pos: Point, geometry: &GeometryStore) -> Point {
        // 1. Grid Snap
        if self.grid_snap_enabled {
            return self.snap_to_grid(pos);
        }
        
        // 2. Object Snap
        if self.object_snap_enabled {
            if let Some(snap_point) = self.find_nearest_snap_point(pos, geometry) {
                return snap_point;
            }
        }
        
        pos
    }
}
```

---

## 📊 コマンド実装優先度

| 優先度 | コマンド | 理由 |
|--------|---------|------|
| **P0** | Line, Circle, Rectangle | 基本中の基本 |
| **P1** | Polyline, Arc, Move, Copy | 実用上必須 |
| **P2** | Offset, Trim, Extend, Fillet | 編集の要 |
| **P3** | Polygon, Spline, Mirror, Rotate | 高度な作図 |
| **P4** | Hatch, Array, Constraints | 専門的機能 |

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-26*
