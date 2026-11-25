use bevy::{
    prelude::*,
    input::mouse::{MouseMotion, MouseWheel},
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
    window::{CursorGrabMode, PrimaryWindow, PresentMode},
};
use std::{collections::{BTreeSet, HashMap}, fs};

const SIZE: i32 = 128;
const HEIGHT: i32 = 48;
const CHUNK: i32 = 16;
const AIR: u8 = 0;
const GRASS: u8 = 1;
const DIRT: u8 = 2;
const STONE: u8 = 3;
const WOOD: u8 = 4;
const LEAVES: u8 = 5;
const SAND: u8 = 6;
const BRICK: u8 = 7;
const BLOCKS: [&str; 7] = ["Grass", "Dirt", "Stone", "Wood", "Leaves", "Sand", "Brick"];
const SAVE: &str = "hpc-world.bin";
const EYE: f32 = 1.62;
const BODY: f32 = 1.80;
const RADIUS: f32 = 0.29;

#[derive(Resource)]
struct Voxels { cells: Vec<u8>, dirty: BTreeSet<(i32,i32)> }
impl Voxels {
    fn index(p: IVec3) -> Option<usize> {
        if p.x < 0 || p.x >= SIZE || p.z < 0 || p.z >= SIZE || p.y < 0 || p.y >= HEIGHT { return None; }
        Some((p.x + SIZE * (p.z + SIZE * p.y)) as usize)
    }
    fn get(&self, p: IVec3) -> u8 { Self::index(p).map_or(AIR, |i| self.cells[i]) }
    fn raw_set(&mut self, p: IVec3, block: u8) { if let Some(i) = Self::index(p) { self.cells[i] = block; } }
    fn set(&mut self, p: IVec3, block: u8) {
        if Self::index(p).is_none() { return; }
        self.raw_set(p, block);
        for d in [IVec3::ZERO, IVec3::X, IVec3::NEG_X, IVec3::Z, IVec3::NEG_Z] {
            let q = p + d;
            if Self::index(q).is_some() { self.dirty.insert((q.x / CHUNK,q.z / CHUNK)); }
        }
    }
    fn mark_all(&mut self) { for x in 0..SIZE/CHUNK { for z in 0..SIZE/CHUNK { self.dirty.insert((x,z)); } } }
    fn generate() -> Self {
        let mut w = Self { cells: vec![0; (SIZE*SIZE*HEIGHT) as usize], dirty: BTreeSet::new() };
        for x in 0..SIZE { for z in 0..SIZE {
            let h = terrain(x,z);
            for y in 0..=h {
                let cave = y > 2 && y < h-3 && (x as f32*0.22+y as f32*0.31).sin() + (z as f32*0.19-y as f32*0.26).cos() + ((x+z) as f32*0.12).sin() > 2.12;
                if cave { continue; }
                let b = if h < 11 && y >= h-2 { SAND } else if y == h { GRASS } else if y >= h-3 { DIRT } else { STONE };
                w.raw_set(IVec3::new(x,y,z),b);
            }
        }}
        for x in 3..SIZE-3 { for z in 3..SIZE-3 {
            if (x-SIZE/2).abs() < 5 && (z-SIZE/2).abs() < 5 { continue; }
            let h = terrain(x,z);
            if h < 11 || hash(x,z) % 1000 >= 12 { continue; }
            let trunk = 3 + (hash(z,x)%2) as i32;
            for dy in 1..=trunk { w.raw_set(IVec3::new(x,h+dy,z),WOOD); }
            for dy in -1_i32..=2 { for dx in -2_i32..=2 { for dz in -2_i32..=2 {
                if dx.abs()+dz.abs()+dy.abs() > 4 || (dy==2 && dx.abs()+dz.abs()>1) { continue; }
                let p = IVec3::new(x+dx,h+trunk+dy,z+dz);
                if w.get(p)==AIR { w.raw_set(p,LEAVES); }
            }}}
        }}
        w.mark_all(); w
    }
    fn save(&self) -> std::io::Result<()> {
        let mut data = b"HPCV01\n".to_vec(); data.extend_from_slice(&self.cells); fs::write(SAVE,data)
    }
    fn load() -> Option<Self> {
        let data = fs::read(SAVE).ok()?;
        let cells = data.strip_prefix(b"HPCV01\n")?;
        if cells.len() != (SIZE*SIZE*HEIGHT) as usize || cells.iter().any(|&b| b > BRICK) { return None; }
        let mut w = Self { cells: cells.to_vec(), dirty: BTreeSet::new() }; w.mark_all(); Some(w)
    }
    fn spawn_feet(&self) -> Vec3 {
        let x=SIZE/2; let z=SIZE/2;
        let top=(0..HEIGHT).rev().find(|&y| self.get(IVec3::new(x,y,z))!=AIR).unwrap_or(0);
        Vec3::new(x as f32+0.5,top as f32+1.02,z as f32+0.5)
    }
}
fn hash(x:i32,z:i32)->u32 {
    let mut n=(x as u32).wrapping_mul(374761393).wrapping_add((z as u32).wrapping_mul(668265263)).wrapping_add(83129);
    n=(n^(n>>13)).wrapping_mul(1274126177); n^(n>>16)
}
fn terrain(x:i32,z:i32)->i32 {
    let x=x as f32; let z=z as f32;
    (14.0+4.0*(x*0.075).sin()+3.5*(z*0.095).cos()+2.0*((x+z)*0.14).sin()+1.1*((x-z)*0.26).cos()).round() as i32
}

// Each face is wound counterclockwise when viewed from outside.
const FACES: [([i32;3], [[f32;3];4]);6] = [
    ([1,0,0], [[1.,0.,0.],[1.,1.,0.],[1.,1.,1.],[1.,0.,1.]]),
    ([-1,0,0], [[0.,0.,0.],[0.,0.,1.],[0.,1.,1.],[0.,1.,0.]]),
    ([0,1,0], [[0.,1.,0.],[0.,1.,1.],[1.,1.,1.],[1.,1.,0.]]),
    ([0,-1,0], [[0.,0.,0.],[1.,0.,0.],[1.,0.,1.],[0.,0.,1.]]),
    ([0,0,1], [[0.,0.,1.],[1.,0.,1.],[1.,1.,1.],[0.,1.,1.]]),
    ([0,0,-1], [[0.,0.,0.],[0.,1.,0.],[1.,1.,0.],[1.,0.,0.]]),
];
fn color(b:u8, n:[i32;3], p:IVec3)->[f32;4] {
    let rgb: [f32;3] = match b {
        GRASS if n[1]==1 => [0.20,0.53,0.11],
        GRASS if n[1]==0 => [0.32,0.38,0.12],
        GRASS | DIRT => [0.36,0.20,0.10],
        STONE => [0.37,0.39,0.41],
        WOOD if n[1]!=0 => [0.54,0.36,0.15],
        WOOD => [0.26,0.13,0.055],
        LEAVES => [0.095,0.34,0.075],
        SAND => [0.77,0.65,0.34],
        BRICK => [0.55,0.15,0.09],
        _ => [1.,1.,1.],
    };
    let shade = 0.86 + (hash(p.x+p.y*91,p.z)%100) as f32*0.0022;
    [rgb[0]*shade,rgb[1]*shade,rgb[2]*shade,1.]
}
fn chunk_mesh(w:&Voxels,cx:i32,cz:i32)->Option<Mesh> {
    let mut pos=Vec::new(); let mut normals=Vec::new(); let mut colors=Vec::new(); let mut indices=Vec::new();
    for y in 0..HEIGHT { for z in 0..CHUNK { for x in 0..CHUNK {
        let p=IVec3::new(cx*CHUNK+x,y,cz*CHUNK+z); let b=w.get(p); if b==AIR { continue; }
        for (n,vertices) in FACES {
            if w.get(p+IVec3::from_array(n))!=AIR { continue; }
            let base=pos.len() as u32; let c=color(b,n,p);
            for v in vertices {
                pos.push([x as f32+v[0],y as f32+v[1],z as f32+v[2]]);
                normals.push([n[0] as f32,n[1] as f32,n[2] as f32]); colors.push(c);
            }
            indices.extend_from_slice(&[base,base+1,base+2,base,base+2,base+3]);
        }
    }}}
    if pos.is_empty() { return None; }
    let mut mesh=Mesh::new(PrimitiveTopology::TriangleList,RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION,pos); mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL,normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR,colors); mesh.insert_indices(Indices::U32(indices)); Some(mesh)
}
#[derive(Resource)]
struct ChunkDraw { material:Handle<StandardMaterial>, chunks:HashMap<(i32,i32),(Entity,Handle<Mesh>)> }
#[derive(Resource)]
struct Player { feet:Vec3, yaw:f32, pitch:f32, vy:f32, grounded:bool, fly:bool, captured:bool, selected:u8, notice:String }
#[derive(Component)] struct View;
#[derive(Component)] struct Hud;
#[derive(Component)] struct Aim;

fn setup(mut commands:Commands, w:Res<Voxels>, mut materials:ResMut<Assets<StandardMaterial>>) {
    let feet=w.spawn_feet();
    commands.insert_resource(Player { feet,yaw:0.,pitch:-0.18,vy:0.,grounded:false,fly:false,captured:false,selected:GRASS,notice:"Click to play".into() });
    commands.insert_resource(ChunkDraw { material:materials.add(StandardMaterial { base_color:Color::WHITE, perceptual_roughness:1.0, ..default() }),chunks:HashMap::new() });
    commands.spawn((Camera3d::default(),Transform::from_translation(feet+Vec3::Y*EYE).with_rotation(Quat::from_rotation_x(-0.18)),View));
    commands.insert_resource(AmbientLight { color:Color::WHITE, brightness:650., ..default() });
    commands.spawn((DirectionalLight { illuminance:11000., shadows_enabled:false, ..default() },Transform::from_xyz(20.,40.,15.).looking_at(Vec3::ZERO,Vec3::Y)));
    commands.spawn((Text::new("HPC CRAFT / RUST EDITION"),TextFont { font_size:23.,..default() },TextColor(Color::WHITE),Node { position_type:PositionType::Absolute,top:Val::Px(16.),left:Val::Px(18.),..default() }));
    commands.spawn((Text::new(""),TextFont { font_size:18.,..default() },TextColor(Color::WHITE),Node { position_type:PositionType::Absolute,bottom:Val::Px(20.),left:Val::Px(18.),padding:UiRect::all(Val::Px(12.)),..default() },BackgroundColor(Color::srgba(0.025,0.04,0.06,0.80)),Hud));
    // Crosshair anchored to the center of the window.
    commands.spawn((Node { position_type:PositionType::Absolute,width:Val::Percent(100.),height:Val::Percent(100.),align_items:AlignItems::Center,justify_content:JustifyContent::Center,..default() })).with_children(|p| { p.spawn((Text::new("+"),TextFont { font_size:26.,..default() },TextColor(Color::WHITE),Aim)); });
}
fn rebuild(mut commands:Commands, mut w:ResMut<Voxels>, mut draw:ResMut<ChunkDraw>, mut meshes:ResMut<Assets<Mesh>>) {
    let dirty=std::mem::take(&mut w.dirty);
    for (cx,cz) in dirty {
        if let Some((entity,handle))=draw.chunks.remove(&(cx,cz)) { commands.entity(entity).despawn(); meshes.remove(handle.id()); }
        if let Some(mesh)=chunk_mesh(&w,cx,cz) {
            let handle=meshes.add(mesh);
            let entity=commands.spawn((Mesh3d(handle.clone()),MeshMaterial3d(draw.material.clone()),Transform::from_xyz((cx*CHUNK) as f32,0.,(cz*CHUNK) as f32))).id();
            draw.chunks.insert((cx,cz),(entity,handle));
        }
    }
}
fn overlaps(feet:Vec3, cell:IVec3)->bool {
    let min=feet-Vec3::new(RADIUS,0.,RADIUS); let max=feet+Vec3::new(RADIUS,BODY,RADIUS); let c=cell.as_vec3();
    min.x<c.x+1. && max.x>c.x && min.y<c.y+1. && max.y>c.y && min.z<c.z+1. && max.z>c.z
}
fn blocked(w:&Voxels,feet:Vec3)->bool {
    if feet.x<RADIUS || feet.z<RADIUS || feet.x>SIZE as f32-RADIUS || feet.z>SIZE as f32-RADIUS || feet.y<0. {return true;}
    let a=(feet-Vec3::new(RADIUS,0.,RADIUS)).floor().as_ivec3();
    let b=(feet+Vec3::new(RADIUS,BODY,RADIUS)).floor().as_ivec3();
    for x in a.x..=b.x {for y in a.y..=b.y {for z in a.z..=b.z {let c=IVec3::new(x,y,z); if w.get(c)!=AIR && overlaps(feet,c) {return true;}}}}
    false
}
fn move_axis(w:&Voxels,feet:&mut Vec3, delta:Vec3)->bool {
    if !blocked(w,*feet+delta) { *feet+=delta; return false; }
    let (mut lo,mut hi)=(0.,1.);
    for _ in 0..9 {let mid=(lo+hi)*0.5; if blocked(w,*feet+delta*mid) {hi=mid;} else {lo=mid;}}
    *feet+=delta*lo; true
}
// Exact grid traversal. Returns the first solid voxel and its neighboring air cell.
fn ray(w:&Voxels, origin:Vec3, dir:Vec3)->Option<(IVec3,Option<IVec3>)> {
    let mut c=origin.floor().as_ivec3();
    let mut step=[0;3]; let mut t=[f32::INFINITY;3]; let mut dt=[f32::INFINITY;3];
    for i in 0..3 { if dir[i].abs()>0.000001 {
        step[i]=if dir[i]>0. {1} else {-1};
        let boundary=c[i] as f32+if step[i]>0 {1.} else {0.};
        t[i]=(boundary-origin[i])/dir[i]; dt[i]=1./dir[i].abs();
    }}
    let mut prev=None;
    for _ in 0..32 {
        if w.get(c)!=AIR {return Some((c,prev));}
        let a=if t[0]<=t[1] && t[0]<=t[2] {0} else if t[1]<=t[2] {1} else {2};
        if t[a]>6. {break;} prev=Some(c); c[a]+=step[a]; t[a]+=dt[a];
    }
    None
}
fn play(time:Res<Time>, keys:Res<ButtonInput<KeyCode>>, mouse:Res<ButtonInput<MouseButton>>, mut motion:EventReader<MouseMotion>, mut wheel:EventReader<MouseWheel>, mut windows:Query<&mut Window,With<PrimaryWindow>>, mut p:ResMut<Player>, mut w:ResMut<Voxels>, mut camera:Query<&mut Transform,With<View>>) {
    let Ok(mut window)=windows.get_single_mut() else {return;};
    let Ok(mut cam)=camera.get_single_mut() else {return;};
    let movement=motion.read().fold(Vec2::ZERO,|acc,e|acc+e.delta);
    let scroll=wheel.read().fold(0.,|acc,e|acc+e.y);
    if keys.just_pressed(KeyCode::F5) {p.notice=match w.save(){Ok(())=>"World saved".into(),Err(e)=>format!("Save failed: {e}")};}
    if keys.just_pressed(KeyCode::F9) {if let Some(loaded)=Voxels::load(){*w=loaded;p.feet=w.spawn_feet();p.vy=0.;p.notice="World loaded".into();}else{p.notice="No valid save found".into();}}
    if keys.just_pressed(KeyCode::Escape) || !window.focused { p.captured=false;window.cursor_options.visible=true;window.cursor_options.grab_mode=CursorGrabMode::None; }
    let mut just_captured=false;
    if !p.captured && window.focused && !keys.just_pressed(KeyCode::Escape) && mouse.just_pressed(MouseButton::Left) { p.captured=true;just_captured=true;window.cursor_options.visible=false;window.cursor_options.grab_mode=CursorGrabMode::Locked;p.notice="".into(); }
    if p.captured {
        if !just_captured {p.yaw-=movement.x*0.0025;p.pitch=(p.pitch-movement.y*0.0025).clamp(-1.54,1.54);}
        for (i,key) in [KeyCode::Digit1,KeyCode::Digit2,KeyCode::Digit3,KeyCode::Digit4,KeyCode::Digit5,KeyCode::Digit6,KeyCode::Digit7].into_iter().enumerate() {if keys.just_pressed(key){p.selected=(i+1) as u8;}}
        if scroll.abs()>0.01 {p.selected=((p.selected as i32-1-scroll.signum() as i32).rem_euclid(7)+1) as u8;}
        if keys.just_pressed(KeyCode::KeyF) {p.fly=!p.fly;p.vy=0.;}
        if keys.just_pressed(KeyCode::KeyR) {p.feet=w.spawn_feet();p.vy=0.;}
        let forward=Quat::from_rotation_y(p.yaw)*Vec3::NEG_Z; let right=Quat::from_rotation_y(p.yaw)*Vec3::X;
        let mut dir=Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW){dir+=forward;} if keys.pressed(KeyCode::KeyS){dir-=forward;} if keys.pressed(KeyCode::KeyD){dir+=right;} if keys.pressed(KeyCode::KeyA){dir-=right;}
        let speed=if keys.pressed(KeyCode::ControlLeft){8.5}else{5.0};
        let velocity=dir.normalize_or_zero()*speed;
        if p.fly {p.vy=if keys.pressed(KeyCode::Space){7.}else if keys.pressed(KeyCode::ShiftLeft){-7.}else{0.};}
        else if p.grounded && keys.just_pressed(KeyCode::Space){p.vy=8.2;p.grounded=false;}
        let dt=time.delta_secs().min(0.10); let steps=(dt/0.0125).ceil().max(1.) as usize; let h=dt/steps as f32;
        for _ in 0..steps {
            if !p.fly {p.vy=(p.vy-24.*h).max(-28.);}
            let dy=p.vy*h;
            move_axis(&w,&mut p.feet,Vec3::X*velocity.x*h);move_axis(&w,&mut p.feet,Vec3::Z*velocity.z*h);
            p.grounded=false;
            if move_axis(&w,&mut p.feet,Vec3::Y*dy) {p.grounded=dy<0.;p.vy=0.;}
        }
        if p.feet.y>HEIGHT as f32+20. {p.feet.y=HEIGHT as f32+20.;}
        if p.feet.y< -3. {p.feet=w.spawn_feet();p.vy=0.;}
        let rotation=Quat::from_rotation_y(p.yaw)*Quat::from_rotation_x(p.pitch);
        if !just_captured && (mouse.just_pressed(MouseButton::Left)||mouse.just_pressed(MouseButton::Right)) {
            if let Some((hit,place))=ray(&w,p.feet+Vec3::Y*EYE,rotation*Vec3::NEG_Z) {
                if mouse.just_pressed(MouseButton::Left) {if hit.y>0 {w.set(hit,AIR);}}
                else if let Some(cell)=place {if w.get(cell)==AIR && !overlaps(p.feet,cell){w.set(cell,p.selected);}}
            }
        }
    }
    cam.translation=p.feet+Vec3::Y*EYE; cam.rotation=Quat::from_rotation_y(p.yaw)*Quat::from_rotation_x(p.pitch);
}
fn hud(p:Res<Player>,mut text:Query<&mut Text,With<Hud>>) {
    if let Ok(mut t)=text.get_single_mut() {
        let bar=BLOCKS.iter().enumerate().map(|(i,b)|if p.selected as usize==i+1 {format!("[{} {}]",i+1,b)}else{format!("{} {}",i+1,b)}).collect::<Vec<_>>().join("   ");
        let mode=if p.fly{"FLY"}else{"WALK"};
        **t=format!("{bar}\n{mode}  |  XYZ {:.0} / {:.0} / {:.0}  |  {}\nWASD move · Space jump/up · Shift down · Ctrl sprint · F fly · R respawn\nLeft break · Right place · 1–7 / wheel select · F5 save · F9 load · Esc release mouse\n{}",p.feet.x,p.feet.y,p.feet.z,if p.captured{"Playing"}else{"Click to capture mouse"},p.notice);
    }
}
fn main() {
    App::new().insert_resource(ClearColor(Color::srgb(0.47,0.73,0.91)))
        .insert_resource(Voxels::load().unwrap_or_else(Voxels::generate))
        .add_plugins(DefaultPlugins.set(WindowPlugin { primary_window:Some(Window { title:"HPC Craft | Rust Edition".into(),resolution:(1280.,720.).into(),present_mode:PresentMode::AutoVsync,..default() }),..default() }))
        .add_systems(Startup,setup).add_systems(Update,(play,rebuild,hud).chain()).run();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn generation_is_repeatable(){let a=Voxels::generate();let b=Voxels::generate();assert_eq!(a.cells,b.cells);assert!(!blocked(&a,a.spawn_feet()));}
    #[test] fn chunk_boundary_edit_marks_both(){let mut w=Voxels::generate();w.dirty.clear();w.set(IVec3::new(16,30,4),WOOD);assert!(w.dirty.contains(&(0,0)));assert!(w.dirty.contains(&(1,0)));}
    #[test] fn ray_hits_first_block(){let mut w=Voxels{cells:vec![0;(SIZE*SIZE*HEIGHT)as usize],dirty:BTreeSet::new()};w.set(IVec3::new(4,3,4),STONE);assert_eq!(ray(&w,Vec3::new(4.5,3.5,7.5),Vec3::NEG_Z),Some((IVec3::new(4,3,4),Some(IVec3::new(4,3,5)))));}
    #[test] fn faces_point_outward(){for (n,v) in FACES {let a=Vec3::from_array(v[0]);let b=Vec3::from_array(v[1]);let c=Vec3::from_array(v[2]);assert!((b-a).cross(c-a).dot(IVec3::from_array(n).as_vec3())>0.);}}
}
