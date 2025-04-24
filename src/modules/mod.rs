use std::{any::Any, collections::HashMap};

use sdl2::{event::Event, image::LoadTexture, pixels::Color, rect::{FPoint, FRect, Point, Rect}, render::{Texture, TextureCreator, WindowCanvas}, video::WindowContext};
use sdl2::keyboard::Keycode;

use crate::game::Game;
// ------------- DEFINIZIONE TRATTI --------------
pub trait GameObject : Any{
    fn as_any(&mut self) -> &mut dyn Any;
    fn draw(&mut self, canvas:&mut WindowCanvas, texture:&Texture, animation_frame:u32) -> Result<(), String>; // restituisco area da disegnare sul canvas
    fn get_name(&self) -> &str;
    fn update(&mut self, deltatime:f32, game_utils:&Utils);
}

// ------------- DEFINIZIONE STRUCTS ed ENUMS -------------

pub enum EntityType {
    Player,
    Enemy,
    Item,
    Other,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PlayerState{
    Idle=0,
    Interaction=1,
    Shoot=2,
    Reload=3,
}

pub struct Utils{
    mouse_position:Point,
}

impl Utils{
    pub fn new() -> Self{
        Utils{
            mouse_position: Point::new(0, 0),
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

    pub fn player_controller(&mut self, event:&Event){
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
            // da spostare in struct utils
            Event::MouseMotion { x, y, xrel, yrel, .. } => {
                //println!("x : {}, y: {}", x, y);
                //println!("x_rel : {}, y_rel: {}", xrel, yrel);
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
    fn draw(&mut self, canvas:&mut WindowCanvas, texture:&Texture, animation_frame:u32) -> Result<(), String> {

        self.player_entity.draw(canvas, texture, self.player_state as u32)?;

        Ok(())
    }

    fn get_name(&self) -> &str {
        self.player_entity.get_name()
    }

    fn update(&mut self, deltatime:f32, game_utils:&Utils) {
        self.player_entity.update(deltatime, game_utils);

        println!("ciao {:?}", game_utils.mouse_position);
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
            
            // imposto rotazione in base alla direzione (posizione (x,y))
            // atan2 ritorna angolo a partire dall'asse x dato il punto (x, y)
            self.rotation = direction_normalized.y.atan2(direction_normalized.x).to_degrees() as f64;

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
}

impl GameObject for Entity{
    fn draw(&mut self, canvas:&mut WindowCanvas, texture:&Texture, animation_frame:u32) -> Result<(), String> {
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
        let entity_screen_position = self.position + 
        FPoint::new(canvas.output_size()?.0 as f32 /2.0, canvas.output_size()?.1 as f32 /2.0);

        // rappresentazione nello schermo dell'entity
        let screen_rect = FRect::from_center(entity_screen_position,
            self.entity_sprite.sprite.width() as f32 * 1.0,
            self.entity_sprite.sprite.height() as f32 * 1.0);
        
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