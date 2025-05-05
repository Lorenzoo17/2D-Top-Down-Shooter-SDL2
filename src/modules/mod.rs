use core::error;
use std::{any::Any, collections::HashMap, vec};

use sdl2::{event::Event, image::LoadTexture, mouse::MouseButton, pixels::Color, rect::{FPoint, FRect, Point, Rect}, render::{Texture, TextureCreator, WindowCanvas}, surface::Surface, video::WindowContext};
use sdl2::keyboard::Keycode;
use sdl2::ttf::Font;
use rand::Rng;
use crate::game::{self, Game};
// ------------- DEFINIZIONE TRATTI --------------
pub trait GameObject : Any{ // gameObject sotto tratto di Any
    fn as_any(&self) -> &dyn Any; // se mi serve solo riferimento immutabile (come per retain)
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn draw(&mut self, canvas:&mut WindowCanvas, texture:&Texture, animation_frame:u32, game_utils:&Utils, scale_factor:f32) -> Result<(), String>; // restituisco area da disegnare sul canvas
    fn get_name(&self) -> &str;
    fn update(&mut self, deltatime:f32, game_utils:&Utils);
    fn is_destroyed(&self) -> bool;
}

pub trait Damageable : GameObject { // tratto che devono implementare tutti gli oggetti che prendono danno, sottotratto, quindi se si implementa Damageable bisogna implementare anche GameObject
    fn take_damage(&mut self, damage:i32);
    fn get_current_health(&self) -> i32;
    fn get_entity(&self) -> &Entity;
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

pub struct EnemySpawner{
    // min e max in quanto nemici vengono spawnati con statistiche casuali in base a cio'
    pub spawn_rate:(f32, f32), // min e max
    pub health_enemies:(i32, i32), // min e max
    pub speed_enemies:(f32, f32), // min e max
    enemy_id:u32,

    current_spawn_rate:f32,
}

impl EnemySpawner{
    pub fn new(enemy_spawn_rate:(f32, f32), health_enemies_range:(i32, i32), speed_enemies_range:(f32, f32)) -> Self{
        EnemySpawner{
            spawn_rate:enemy_spawn_rate,
            current_spawn_rate:enemy_spawn_rate.0,
            health_enemies:health_enemies_range,
            speed_enemies:speed_enemies_range,
            enemy_id:0, // inizialemente a 0 -> ad ogni nemico che viene spawnato si incrementa di 1
        }
    }

    pub fn spawn_enemies(&mut self, deltatime:f32, game_utils:&Utils) -> Option<Vec<Enemy>>{ // movimento vec
        if self.current_spawn_rate <= 0.0{

            // vita e velocita' casuale tra i due range min e max
            let enemy_health = rand::thread_rng().gen_range(self.health_enemies.0..self.health_enemies.1);
            let enemy_speed = rand::thread_rng().gen_range(self.speed_enemies.0..self.speed_enemies.1);

            let enemies_to_spawn = 2; // numero di nemici da spawnare in contemporanea (per ora fisso)
            let mut enemies_spawned:Vec<Enemy> = Vec::new(); // vettore nel quale metterli
            // ognuno spawna con posizione casuale, per ora stessa velocita' e vita
            for _ in 0..enemies_to_spawn{
                let mut new_enemy = Enemy::new(format!("enemy_base_{}", self.enemy_id).as_str(), enemy_speed, enemy_health);
                self.enemy_id += 1; // si incrementa enemy_id
    
                // si imposta sprite
                new_enemy.enemy_entity.set_sprite(51, 43);
                // si imposta posizione (casualizzarla)
                let offset_player_x_min = game_utils.get_player_position().x - 100.0;
                let offset_player_x_max = game_utils.get_player_position().x + 100.0;
                let offset_player_y_min = game_utils.get_player_position().y - 100.0;
                let offset_player_y_max = game_utils.get_player_position().y + 100.0;
                
                let spawn_position_x = rand::thread_rng().gen_range(offset_player_x_min..offset_player_x_max);
                let spawn_position_y = rand::thread_rng().gen_range(offset_player_y_min..offset_player_y_max);
                new_enemy.enemy_entity.set_position(FPoint::new(spawn_position_x, 
                    spawn_position_y));
                
                // spawn rate casuale tra minimo e massimo
                self.current_spawn_rate = rand::thread_rng().gen_range(self.spawn_rate.0..self.spawn_rate.1);

                enemies_spawned.push(new_enemy); // pusho il nuovo nemico
            }
        
            Some(enemies_spawned)
        }else{
            self.current_spawn_rate -= deltatime;
            None
        }
    }

    pub fn spawn_enemy(&mut self, deltatime:f32, game_utils:&Utils) -> Option<Enemy>{ // movimento dell'enemy
        if self.current_spawn_rate <= 0.0{

            // vita e velocita' casuale tra i due range min e max
            let enemy_health = rand::thread_rng().gen_range(self.health_enemies.0..self.health_enemies.1);
            let enemy_speed = rand::thread_rng().gen_range(self.speed_enemies.0..self.speed_enemies.1);
            
            let mut new_enemy = Enemy::new(format!("enemy_base_{}", self.enemy_id).as_str(), enemy_speed, enemy_health);
            self.enemy_id += 1; // si incrementa enemy_id

            // si imposta sprite
            new_enemy.enemy_entity.set_sprite(51, 43);
            // si imposta posizione (casualizzarla)
            let offset_player_x_min = game_utils.get_player_position().x - 100.0;
            let offset_player_x_max = game_utils.get_player_position().x + 100.0;
            let offset_player_y_min = game_utils.get_player_position().y - 100.0;
            let offset_player_y_max = game_utils.get_player_position().y + 100.0;
            
            let spawn_position_x = rand::thread_rng().gen_range(offset_player_x_min..offset_player_x_max);
            let spawn_position_y = rand::thread_rng().gen_range(offset_player_y_min..offset_player_y_max);
            new_enemy.enemy_entity.set_position(FPoint::new(spawn_position_x, 
                spawn_position_y));
            
            // spawn rate casuale tra minimo e massimo
            self.current_spawn_rate = rand::thread_rng().gen_range(self.spawn_rate.0..self.spawn_rate.1);
            Some(new_enemy)
        }else{
            self.current_spawn_rate -= deltatime;
            None
        }
    }

    pub fn increase_difficulty(&mut self){
        if self.spawn_rate.0 > 0.5 {
            self.spawn_rate = (self.spawn_rate.0 - 0.25, self.spawn_rate.1 - 0.25);
        }

        if self.health_enemies.0 < 40 {
            self.health_enemies = (self.health_enemies.0 + 5, self.health_enemies.1 + 5);
        }

        if self.speed_enemies.0 < 70.0 {
            self.speed_enemies = (self.speed_enemies.0 + 5.0, self.speed_enemies.1 + 5.0);
        }
    }
}

pub struct Enemy{
    pub enemy_entity: Entity,
    health:i32,
    current_health:i32,
    speed:f32,
}

impl Enemy{
    pub fn new(name:&str, speed:f32, health:i32) -> Enemy{
        Enemy{
            enemy_entity: Entity::with_speed(name, speed, EntityType::Enemy),
            health:health,
            current_health:health,
            speed:speed,
        }
    }

    // funzione eseguita in game.update() per ogni nemico
    pub fn damage_player(&mut self, player:&mut Player){
        if Utils::calculate_point_distance(self.enemy_entity.position, player.player_entity.get_position()) < 15.0{
            player.take_damage(10); //damage fisso
            println!("Vita player : {}", player.current_health);
            self.current_health = -1; // imposto vita a -1 in modo che poi si autodistrugga
        }
    }
}

impl GameObject for Enemy{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn draw(&mut self, canvas:&mut WindowCanvas, texture:&Texture, animation_frame:u32, game_utils:&Utils, scale_factor:f32) -> Result<(), String> {
        self.enemy_entity.draw(canvas, texture, animation_frame, game_utils, scale_factor)
    }

    fn get_name(&self) -> &str {
        self.enemy_entity.get_name()
    }

    fn update(&mut self, deltatime:f32, game_utils:&Utils) {
        // calcolare la movement_direction in base a posizione del player
        // eseguire questo update solo se player e al di sotto di una certa distanza

        // prendo direzione verso il player (target - self)
        let target_direction = game_utils.get_player_position() - self.enemy_entity.get_position();
        // normalizzo la direzione
        let target_direction_normalized = Utils::point_normalized(target_direction);
        // setto la nuova direzione del nemico verso il player
        self.enemy_entity.change_direction(target_direction_normalized); 

        // si controlla vicinanza con player
        if Utils::calculate_point_distance(self.enemy_entity.position, 
            game_utils.get_player_position()) > 15.0{ // se player e' massimo lontano 15 pixel
            self.enemy_entity.update(deltatime, game_utils); // update di base dell'entity per movimento sulla base di movement_direction
        }
    }

    fn is_destroyed(&self) -> bool {
        if self.current_health <= 0{
            true
        }else{
            false
        }
    }
}

impl Damageable for Enemy{
    fn get_current_health(&self) -> i32 {
        self.current_health
    }

    fn take_damage(&mut self, damage:i32) {
        self.current_health -= damage;
    }

    fn get_entity(&self) -> &Entity {
        &self.enemy_entity
    }
}

pub struct Bullet{
    bullet_entity:Entity, // entity relativa al bullet, contiene di base velocita', direzione, nome ecc..
    bullet_owner: EntityType, // assegnato alla creazione, per capire chi ha sparato il proiettile
    bullet_life:i32, // vita del bullet in base alla distanza percorsa
    bullet_current_life:i32, // attuale distanza percorsa
    destroyed:bool, // si imposta a true quando ad esempio colpisce nemico o ostacolo
    bullet_damage:i32,
}

impl Bullet{
    pub fn new(bullet_direction:FPoint, bullet_owner:EntityType, bullet_velocity:f32, bullet_starting_position:FPoint) -> Self{
        let mut new_bullet = Bullet{
            bullet_entity: Entity::with_speed("bullet", bullet_velocity, EntityType::Bullet),
            bullet_owner:bullet_owner,
            bullet_life:200, // di base 200 pixel
            bullet_current_life:0,
            destroyed:false,
            bullet_damage:10, // per ora fisso
        };

        new_bullet.bullet_entity.change_direction(bullet_direction); // imposto la direzione al nuovo bullet creato
        new_bullet.bullet_entity.set_position(bullet_starting_position); // posizione in cui istanziare il bullet

        new_bullet.bullet_entity.set_sprite(100,50); // imposto la dimensione dello sprite (coincide con dimensione missile.png per ora)
        // la texture poi si specifica quando si renderizza direttamente, quindi in Game.render()

        new_bullet
    }

    pub fn is_out_of_range(&self) -> bool{
        if self.bullet_current_life >= self.bullet_life{
            true
        }else{
            false
        }
    }

    // metodo generico in modo da colpire qualsiasi entita' damageable SOLO SE il bullet owner e' diverso dall'entity type
    pub fn damage_enemy<T>(&mut self, enemy:&mut T) where T : Damageable{ // metodo che viene eseguito nell'update di game.rs per ogni bullet
        // se il bullet e' vicino al nemico i-esimo
        let bullet_range:f32 = 20.0; // vicinanza in pixel tra bullet e nemico per far si che il bullet possa colpirlo
        if Utils::calculate_point_distance(self.bullet_entity.get_position(), enemy.get_entity().get_position()) < bullet_range{
            if enemy.get_entity().entity_type != self.bullet_owner{
                enemy.take_damage(self.bullet_damage); // damage fisso
                println!("{} health : {}", enemy.get_entity().entity_name, enemy.get_current_health()); // stampo la vita rimanente

                self.bullet_damage = 0; // per evitare problemi (distrutto troppo tardi)
                self.destroyed = true; // distruggo il bullet
            }
        }
    }
}

impl GameObject for Bullet{
    fn get_name(&self) -> &str {
        self.bullet_entity.get_name()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn draw(&mut self, canvas:&mut WindowCanvas, texture:&Texture, animation_frame:u32, game_utils:&Utils, scale_factor:f32) -> Result<(), String> {
        self.bullet_entity.draw(canvas, texture, animation_frame, game_utils, scale_factor)
    }

    fn update(&mut self, deltatime:f32, game_utils:&Utils) {
        self.bullet_entity.update(deltatime, game_utils); // ho il movimento gia' gestito di base da Entity

        self.bullet_current_life += (self.bullet_entity.speed * deltatime) as i32; // aggiorno la distanza percorsa
    }

    fn is_destroyed(&self) -> bool {
        self.destroyed || self.is_out_of_range() // per bullet OR tra out_of_range e destroyed
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
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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

    fn is_destroyed(&self) -> bool {
        false
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

    // funzioni "statiche" quindi senza self, utilizzata per fare operazioni di utilita' generica
    // possono essere richiamate semplicemente con Utils::point_magnitude()
    // non avendo self non dipendono dall'istanza utils creata e, non ritornando Self, non ne creano una
    pub fn point_magnitude(vector:FPoint) -> f32{
        (vector.x * vector.x + vector.y * vector.y).sqrt()
    }

    pub fn point_normalized(vector:FPoint) -> FPoint{
        let vector_magnitude = Utils::point_magnitude(vector);

        if vector_magnitude > 0.0 {
            vector / vector_magnitude
        }else{
            vector
        }
    }

    pub fn calculate_point_distance(v1:FPoint, v2:FPoint) -> f32{
        // semplice distanza tra 2 punti
        ((v2.x - v1.x) * (v2.x - v1.x) + (v2.y - v1.y) * (v2.y - v1.y)).sqrt()
    }

    // scrittura
    pub fn write_on_screen(to_write:&str, canvas:&mut WindowCanvas, font:&Font, resource_manager:&ResourceManager, pos:Point, size:Point) -> Result<(), String>{
        let surface = font.render(to_write)
        .blended(Color::RGB(255, 255, 255)).map_err(|error| {
            error.to_string()
        })?; // si propaga errore

        // si crea la texture "dell'immagine" creata con il font
        let font_texture = resource_manager.get_texture_from_surface(surface)?;

            // creo il target come Rect, che specifica la posizione e la grandezza della zona in cui voglio spiaccicare
        // la texture
        let target = Rect::new(pos.x, pos.y, size.x as u32, size.y as u32);
        // faccio canvas.copy() per mettere la scritta nel canvas
        // (texture da mettere, None = voglio mettere tutta la texture, Dove e quanto grande deve essere)
        canvas.copy(&font_texture, None, Some(target))?;

        Ok(())
    }
}

pub struct Player{
    pub player_entity:Entity,
    pub player_state:PlayerState,
    fire_rate: f32, // rate di sparo dei proiettili
    current_fire_rate: f32,
    health:i32,
    current_health:i32,
}

impl Player{
    pub fn new(name:&str, _speed:f32, _health:i32) -> Self{
        Player { player_entity: Entity::with_speed(name, _speed, EntityType::Player),
        player_state:PlayerState::Idle,
        fire_rate: 0.5, // fire rate di base
        current_fire_rate: 0.0,
        health: _health,
        current_health: _health, }
    }

    pub fn with_fire_rate(name:&str, _speed:f32, _health:i32, initial_fire_rate:f32) -> Self{
        Player { player_entity: Entity::with_speed(name, _speed, EntityType::Player),
            player_state:PlayerState::Idle,
            fire_rate: initial_fire_rate,
            current_fire_rate: 0.0,
            health: _health,
            current_health: _health, }
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

                    if self.current_fire_rate <= 0.0{ // solo se posso sparare 
                    
                    // si crea nuovo bullet
                    let bullet_velocity = 200.0;
                    let bullet_direction = self.player_entity.get_forward_direction();
                    
                    // per spostare la posizione del bullet rispetto al sistema di riferimento locale del player
                    // devo ottenere il suo S.R.L 
                    // ottengo quindi il forward che sarebbe la direzione in cui punta il player
                    // ottengo l'asse right mediante rotazione di 90 gradi in senso orario
                    let forward = self.player_entity.get_forward_direction();
                    let right = self.player_entity.get_right_direction();
                    let bullet_offset = forward * 20.0 + right * -12.0; // sposto in avanti di 20 pixel e a sinistra di 12
                    
                    // quindi parto dal player e sposto il bullet dell'offset, calcolato nel S.R.L del player, quindi
                    // quando il player si sposta rimane invariato
                    let bullet_starting_position = self.player_entity.position + bullet_offset;

                    let new_bullet = Bullet::new(bullet_direction
                        , self.player_entity.entity_type, bullet_velocity, bullet_starting_position);
                    
                    // si mette bullet nella lista dei gameobjects di game
                    gameobjects_list.push(Box::new(new_bullet)); // si crea una copia nello heap di new_bullet
                    // alla fine del metodo new_bullet dichiarato qui su viene automaticamente droppato, mentre quello nella gameobjects_list rimane appunto nello heap
                    
                    self.current_fire_rate = self.fire_rate; // resetto il current_fire_rate 
                    }
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

    pub fn set_fire_rate(&mut self, new_fire_rate:f32){
        self.fire_rate = new_fire_rate;
    }

    pub fn get_fire_rate(&self) -> f32{
        self.fire_rate
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

        if self.current_fire_rate > 0.0{
            self.current_fire_rate -= deltatime;
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn is_destroyed(&self) -> bool {
        if self.get_current_health() <= 0{
            true
        }else{
            false
        }
    }
}

impl Damageable for Player{
    fn get_current_health(&self) -> i32 {
        self.current_health
    }

    fn take_damage(&mut self, damage:i32) {
        self.current_health -= damage;
    }

    fn get_entity(&self) -> &Entity {
        &self.player_entity
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
            let direction_normalized = Utils::point_normalized(self.movement_direction);
            
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

    pub fn get_right_direction(&self) -> FPoint{
        FPoint::new(self.get_forward_direction().y, -self.get_forward_direction().x)
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn is_destroyed(&self) -> bool {
        false
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

    pub fn get_texture_from_surface(&self, surface: Surface) -> Result<Texture, String>{
        let surface_texture = self.texture_creator.create_texture_from_surface(surface).map_err(|err|{
            err.to_string()
        })?;

        Ok(surface_texture)
    }
}