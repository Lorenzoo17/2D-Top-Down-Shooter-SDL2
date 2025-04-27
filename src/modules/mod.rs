use std::{any::Any, collections::HashMap};

use sdl2::{event::Event, image::LoadTexture, mouse::MouseButton, pixels::Color, rect::{FPoint, FRect, Point, Rect}, render::{Texture, TextureCreator, WindowCanvas}, video::WindowContext};
use sdl2::keyboard::Keycode;

use crate::game::{self, Game};
// ------------- DEFINIZIONE TRATTI --------------
pub trait GameObject : Any{
    fn as_any(&mut self) -> &mut dyn Any;
    fn draw(&mut self, canvas:&mut WindowCanvas, texture:&Texture, animation_frame:u32, game_utils:&Utils, scale_factor:f32) -> Result<(), String>; // restituisco area da disegnare sul canvas
    fn get_name(&self) -> &str;
    fn update(&mut self, deltatime:f32, game_utils:&Utils);
}

// ------------- DEFINIZIONE STRUCTS ed ENUMS -------------

// si implementa tratto copy per evitare di dover mettere il lifetime in bullet quando si passa EntityType
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum EntityType {
    Player,
    Enemy,
    Item,
    Other,
    Bullet,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PlayerState{
    Idle=0,
    Interaction=1,
    Shoot=2,
    Reload=3,
}

pub struct Bullet{
    bullet_entity:Entity, // entity relativa al bullet, contiene di base velocita', direzione, nome ecc..
    bullet_owner: EntityType, // assegnato alla creazione, per capire chi ha sparato il proiettile
}

impl Bullet{
    pub fn new(bullet_direction:FPoint, bullet_owner:EntityType, bullet_velocity:f32, bullet_starting_position:FPoint) -> Self{
        let mut new_bullet = Bullet{
            bullet_entity: Entity::with_speed("bullet", bullet_velocity, EntityType::Bullet),
            bullet_owner:bullet_owner
        };

        new_bullet.bullet_entity.change_direction(bullet_direction); // imposto la direzione al nuovo bullet creato
        new_bullet.bullet_entity.set_position(bullet_starting_position); // posizione in cui istanziare il bullet

        new_bullet.bullet_entity.set_sprite(100,50); // imposto la dimensione dello sprite (coincide con dimensione missile.png per ora)
        // la texture poi si specifica quando si renderizza direttamente, quindi in Game.render()

        new_bullet
    }
}

impl GameObject for Bullet{
    fn get_name(&self) -> &str {
        self.bullet_entity.get_name()
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn draw(&mut self, canvas:&mut WindowCanvas, texture:&Texture, animation_frame:u32, game_utils:&Utils, scale_factor:f32) -> Result<(), String> {
        self.bullet_entity.draw(canvas, texture, animation_frame, game_utils, scale_factor)
    }

    fn update(&mut self, deltatime:f32, game_utils:&Utils) {
        self.bullet_entity.update(deltatime, game_utils); // ho il movimento gia' gestito di base da Entity
    }
}

pub struct Camera{
    camera_position:FPoint,
}

impl Camera{
    pub fn new() -> Self{
        Camera{
            camera_position: FPoint::new(0.0, 0.0),
        }
    }

    pub fn get_main_camera_position(&self) -> FPoint{
        self.camera_position
    }
}

impl GameObject for Camera{
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn get_name(&self) -> &str {
        "main_camera"
    }

    fn draw(&mut self, canvas:&mut WindowCanvas, texture:&Texture, animation_frame:u32, game_utils:&Utils, scale_factor:f32) -> Result<(), String> {
        // nothing to draw for camera
        Ok(())
    }

    fn update(&mut self, deltatime:f32, game_utils:&Utils) {
        // camera segue il player
        // 400, 300 rappresenta la metà della dimensione della finestra -> MODIFICARE PASSANDO IL CANVAS!!!
        self.camera_position = game_utils.get_player_position() - FPoint::new(400.0, 300.0);
    }
}

pub struct Utils{
    mouse_position:Point,
    player_position:FPoint,
    pub main_camera_position:FPoint,
}

impl Utils{
    pub fn new() -> Self{
        Utils{
            mouse_position: Point::new(0, 0),
            player_position: FPoint::new(0.0, 0.0),
            main_camera_position: FPoint::new(0.0, 0.0),
        }
    }

    pub fn utils_manage_events(&mut self, event:&Event) -> (){
        match event{
            Event::MouseMotion { x, y, xrel, yrel , ..} =>{
                self.mouse_position = Point::new(*x, *y); // aggiorno mouse position
            }

            _ => {

            }
        }
    }

    pub fn save_player_position(&mut self, player_position:FPoint) -> (){
        self.player_position = player_position;
    }

    pub fn get_player_position(&self) -> FPoint{
        self.player_position
    }

}

pub struct Player{
    pub player_entity:Entity,
    pub player_state:PlayerState,
}

impl Player{
    pub fn new(name:&str, _speed:f32) -> Self{
        Player { player_entity: Entity::with_speed(name, _speed, EntityType::Player),
        player_state:PlayerState::Idle }
    }

    pub fn player_controller(&mut self, event:&Event, gameobjects_list:&mut Vec<Box<dyn GameObject>>){
        match event{
            Event::KeyDown { keycode:Some(Keycode::F), .. } =>{
                //if self.player_state != PlayerState::Interaction{
                //    self.player_state = PlayerState::Shoot;
                //}
                if self.player_state != PlayerState::Shoot{
                    self.player_state = PlayerState::Shoot;
                }else{
                    self.player_state = PlayerState::Idle; // torno in idle se premo F in shoot
                }
            },
            Event::MouseButtonDown { mouse_btn:MouseButton::Left, .. } => {
                if self.player_state == PlayerState::Shoot{
                    
                    // si crea nuovo bullet
                    let bullet_velocity = 200.0;
                    let bullet_direction = self.player_entity.get_forward_direction();
                    let bullet_starting_position = FPoint::new(self.player_entity.position.x, 
                        self.player_entity.position.y);

                    let new_bullet = Bullet::new(bullet_direction
                        , self.player_entity.entity_type, bullet_velocity, self.player_entity.get_position());
                    
                    // si mette bullet nella lista dei gameobjects di game
                    gameobjects_list.push(Box::new(new_bullet)); // si crea una copia nello heap di new_bullet
                    // alla fine del metodo new_bullet dichiarato qui su viene automaticamente droppato, mentre quello nella gameobjects_list rimane appunto nello heap
                }
            }

            _ => {

            }
        }
    }

    pub fn move_player(&mut self, event:&Event){
        match event {
            Event::KeyDown { keycode:Some(Keycode::A), .. } =>{
                // cambio direzione lungo x e non lungo y
                self.player_entity.change_direction(FPoint::new(-1.0,self.player_entity.movement_direction.y));
            },
            Event::KeyDown { keycode:Some(Keycode::D), .. } =>{
                // cambio direzione lungo x e non lungo y
                self.player_entity.change_direction(FPoint::new(1.0,self.player_entity.movement_direction.y));
            },
            Event::KeyDown { keycode:Some(Keycode::S), .. } =>{
                // cambio direzione lungo x e non lungo y
                self.player_entity.change_direction(FPoint::new(self.player_entity.movement_direction.x, 1.0));
            },
            Event::KeyDown { keycode:Some(Keycode::W), .. } =>{
                // cambio direzione lungo x e non lungo y
                self.player_entity.change_direction(FPoint::new(self.player_entity.movement_direction.x, -1.0));
            },
            Event::KeyUp { keycode:Some(Keycode::A), ..} |
            Event::KeyUp { keycode:Some(Keycode::D), ..} => {
                self.player_entity.change_direction(FPoint::new(0.0, self.player_entity.movement_direction.y));
            },
            Event::KeyUp { keycode:Some(Keycode::S), ..} |
            Event::KeyUp { keycode:Some(Keycode::W), ..} => {
                self.player_entity.change_direction(FPoint::new(self.player_entity.movement_direction.x, 0.0));
            },

            _ => {

            }
        }
    }
}

// Implemento GameObject per player utilizzando le funzioni di player_entity
impl GameObject for Player{
    fn draw(&mut self, canvas:&mut WindowCanvas, texture:&Texture, animation_frame:u32, game_utils:&Utils, scale_factor:f32) -> Result<(), String> {

        self.player_entity.draw(canvas, texture, self.player_state as u32, game_utils, scale_factor)?;

        Ok(())
    }

    fn get_name(&self) -> &str {
        self.player_entity.get_name()
    }

    fn update(&mut self, deltatime:f32, game_utils:&Utils) {
        self.player_entity.update(deltatime, game_utils);

        
        // con il movimento della camera rispetto al player, la posizione del mouse deve essere sommata alla posizione
        // della camera
        let mouse_world_position = FPoint::new(
            game_utils.mouse_position.x as f32 + game_utils.main_camera_position.x,
            game_utils.mouse_position.y as f32 + game_utils.main_camera_position.y,
        );
        
        // posizione del mouse relativa al player -> (target - player) -> poi uso atan2
        let relative_mouse_position = mouse_world_position - self.player_entity.position;
        
        // rotazione sempre mediante atan2
        let player_rotation = (relative_mouse_position.y as f64).atan2((relative_mouse_position.x as f64)).to_degrees();
        self.player_entity.rotation = player_rotation;
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct Entity{
    position:FPoint,
    rotation:f64,
    speed:f32,
    pub movement_direction:FPoint,
    entity_sprite:Sprite,
    entity_name:String,
    entity_type:EntityType
}

impl Entity{
    // passo dimensione dello sprite
    pub fn new(name:&str, _entity_type:EntityType) -> Self{
        Entity { 
            position: FPoint::new(0.0, 0.0), 
            speed: 0.0, 
            movement_direction: FPoint::new(0.0, 0.0), 
            entity_sprite: Sprite::new(10, 10),
            entity_name: name.to_string(),
            entity_type: _entity_type,
            rotation:0.0,
        }
    }

    pub fn with_speed(name:&str, _speed:f32, _entity_type:EntityType) -> Self{
        Entity { 
            position: FPoint::new(0.0, 0.0), 
            speed: _speed, 
            movement_direction: FPoint::new(0.0, 0.0), 
            entity_sprite: Sprite::new(10, 10),
            entity_name: name.to_string(),
            entity_type: _entity_type,
            rotation:0.0,
        }
    }

    pub fn change_direction(&mut self, direction:FPoint){
        self.movement_direction = direction;
    }

    // metodo generico di movimento per tutte le entities. Utilizzare EntittyType per gestire comportamenti diversi in base
    // alla singola entity
    pub fn move_entity(&mut self) -> FPoint{ // ritorno la posizione qui in modo da dover passare il deltatime solo in Update
        if self.movement_direction != FPoint::new(0.0, 0.0){
            let direction_magnitude = (self.movement_direction.x * self.movement_direction.x 
            + self.movement_direction.y * self.movement_direction.y).sqrt();
            
            let direction_normalized = if direction_magnitude > 0.0 { self.movement_direction / direction_magnitude}
            else {self.movement_direction}; // normalizza direzione
            
            // la rotazione fatta in questo modo funziona in automatico per tutti gli sprite che
            // puntano verso destra!!!!
            if self.entity_type != EntityType::Player{ // solo il player non deve ruotare in base alla direzione di spostamento 
                // imposto rotazione in base alla direzione (posizione (x,y))
                // atan2 ritorna angolo a partire dall'asse x dato il punto (x, y)
                self.rotation = direction_normalized.y.atan2(direction_normalized.x).to_degrees() as f64;
            }
        
            self.position + direction_normalized * self.speed // ritorno posizione aggiornata (va poi moltiplicato deltatime)
        }else {
            self.position // non si muove
        }
    }

    pub fn set_sprite(&mut self, frame_width:u32, frame_height:u32){
        self.entity_sprite = Sprite::new(frame_width, frame_height);
    }

    pub fn get_rotation(&self) -> f64{
        self.rotation
    }

    pub fn set_rotation(&mut self, angle:f64) -> (){
        self.rotation = angle;
    }

    pub fn get_position(&self) -> FPoint{
        self.position
    }

    pub fn set_position(&mut self, position:FPoint) -> (){
        self.position = position;
    }

    pub fn get_forward_direction(&self) -> FPoint{
        FPoint::new(self.rotation.to_radians().cos() as f32, self.rotation.to_radians().sin() as f32)
    }
}

impl GameObject for Entity{
    fn draw(&mut self, canvas:&mut WindowCanvas, texture:&Texture, animation_frame:u32,game_utils:&Utils, scale_factor:f32) -> Result<(), String> {
        // dimensione del singolo sprite nello spritesheet
        let (frame_width, frame_height) = self.entity_sprite.sprite.size();

        // sprite nello sprite sheet
        let current_frame_in_sprite_sheet = Rect::new(
            self.entity_sprite.sprite.x() + frame_width as i32 * animation_frame as i32,
            self.entity_sprite.sprite.y() + frame_height as i32  * self.entity_sprite.current_frame,
            frame_width, 
            frame_height,
        );

        // entity parte in posizione (0, 0) che sarebbe in alto a sx. Per farlo partire al centro dello schermo
        // si usa come offset il centro del canvas
        // let entity_screen_position = self.position;
        //+ FPoint::new(canvas.output_size()?.0 as f32 /2.0, canvas.output_size()?.1 as f32 /2.0);

        // sposto nel S.R della camera, quindi sottraggo la posizione della camera alla posizione del gameobject
        // 
        let entity_screen_position = self.position - game_utils.main_camera_position;

        // rappresentazione nello schermo dell'entity
        let screen_rect = FRect::from_center(entity_screen_position,
            self.entity_sprite.sprite.width() as f32 * scale_factor,
            self.entity_sprite.sprite.height() as f32 * scale_factor);
        
        // converto in intero
        let output_rect = Rect::new(
            screen_rect.x.round() as i32,
            screen_rect.y.round() as i32,
            screen_rect.width().round() as u32,
            screen_rect.height().round() as u32,
        );

        // utilizzare copy_ex se angle != 0.
        // come center fare center:Point = Point::new(output_rect_widht/2, output_rect_height/2)
        let sprite_center:Point = Point::new(output_rect.width() as i32 / 2, output_rect.height() as i32 / 2);

        canvas.copy_ex(texture, current_frame_in_sprite_sheet, output_rect, 
            self.get_rotation(), sprite_center, false, false)?;


        // DEBUG

        let debug_sprite_center = true;
        if debug_sprite_center{
            // si crea un punto e si mette nella posizione dello sprite in output (+ centro sprite)
            // output_rect dipende dalla posizione dell'entity, quindi si sposta in automatico
            let debug_point = Point::new(
                output_rect.x + sprite_center.x,
                output_rect.y + sprite_center.y,
            );
            canvas.set_draw_color(Color::RGB(255, 0, 0)); // si setta colore del prossimo disegno (pipeline tipo opengl)
            let debug_rect = Rect::new(debug_point.x, debug_point.y, 3, 3); // si setta rect da disegnare per rappresentare il centro
            canvas.fill_rect(debug_rect)?; // si disegna il rect
        }

        Ok(())
    }

    fn get_name(&self) -> &str {
        self.entity_name.as_str()
    }

    fn update(&mut self, deltatime:f32, game_utils:&Utils){
        let movement = (self.move_entity() - self.position) * deltatime;
        self.position = self.position + movement;

        // eventuale gestione animazione
        // .. 
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct Sprite{
    sprite:Rect, // rect in quanto rappresenta zona da disegnare sul canvas
    current_frame: i32,
}

impl Sprite{
    // specifico dimensione zona da disegnare sulla base della grandezza del singolo frame
    // nello spritesheet di riferimento
    pub fn new(sprite_x_dimension:u32, sprite_y_dimension:u32) -> Self{
        Sprite { 
            sprite: Rect::new(0, 0, sprite_x_dimension, sprite_y_dimension),
            current_frame: 0, 
        }
    }
}

// per contenere tutte le textures, contiene anche il texture creator per caricarle e restituirle
pub struct ResourceManager<'l>{
    texture_creator:&'l TextureCreator<WindowContext>,
    textures: HashMap<String, Texture<'l>>,
}

impl<'l> ResourceManager<'l>{
    // passo il texture_creator quando inizializzo Game.rs
    pub fn new(texture_creator: &'l TextureCreator<WindowContext>) -> Self {
        Self {
            texture_creator,
            textures: HashMap::new(),
        }
    }

    pub fn load_texture(&mut self, nome:&str, path:&str) -> Result<(), String>{
        let texture = self.texture_creator.load_texture(path).expect("Errore caricamento texture");

        if self.textures.contains_key(nome) == false{
            self.textures.insert(nome.to_string(), texture);
        }else{
            println!("Texture con nome {} gia' inserita", nome);
        }

        Ok(())
    }

    pub fn get_texture(&self, nome:&str) -> Option<&Texture<'l>>{
        self.textures.get(nome) // riferimento non mutabile alla texture ovviamente, non si vuole modificare
    } 
}