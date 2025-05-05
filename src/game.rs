use std::path::Path;

use sdl2::{image::LoadTexture, keyboard::Keycode, rect::{FPoint, Point}, render::{Canvas, TextureCreator, WindowCanvas}, video::WindowContext, EventPump, Sdl};
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::ttf::Font;

use crate::modules::{Bullet, Camera, Enemy, Entity, EntityType, GameObject, Player, ResourceManager, Utils, Damageable,
EnemySpawner};

pub struct Game<'l>{
    canvas: &'l mut WindowCanvas,
    event_pump: &'l mut EventPump,
    resource_manager: ResourceManager<'l>,
    player: Player,
    gameobjects:Vec<Box<dyn GameObject>>,
    main_camera:Camera,
    enemy_spawner:EnemySpawner,
    game_score:i32,
    last_score_checkpoint: i32, // nuova variabile per evitare che la difficolta' incrementi quando lo score e' fermo a 50

    utils:Utils,
}

impl<'l> Game<'l>{
    // ritorno result in quanto per creare canvas ecc necessito di propagare l'errore, vale comunque come costruttore
    pub fn new(canvas_main:&'l mut WindowCanvas, texture_creator:&'l TextureCreator<WindowContext>, event_pump_main:&'l mut EventPump) -> Result<Self, String>{
        let mut resources = ResourceManager::new(texture_creator);
        
        // qui carico tutte le textures
        resources.load_texture("player", "assets/survivor_sheet.png").expect("Errore caricamento textures");
        resources.load_texture("default", "assets/spritesheet_characters.png").expect("Errore caricamento textures");
        resources.load_texture("bullet", "assets/missile.png").expect("Errore caricamento texture missile");

        let mut player = Player::new("Player", 50.0, 100);
        player.player_entity.set_sprite(51, 43);

        // enemy di prova per testare il rendering
        let mut base_enemy:Enemy = Enemy::new("Enemy_0", 25.0, 10);
        base_enemy.enemy_entity.set_sprite(51, 43);
        base_enemy.enemy_entity.set_position(FPoint::new(40.0, 40.0));

        let mut base_enemy_2:Enemy = Enemy::new("Enemy_1", 25.0, 10);
        base_enemy_2.enemy_entity.set_sprite(51, 43);
        base_enemy_2.enemy_entity.set_position(FPoint::new(-80.0, -80.0));

        let mut gameobjects_list:Vec<Box<dyn GameObject>> = Vec::new(); // si crea la lista di gameobjects
        gameobjects_list.push(Box::new(base_enemy)); // si crea un puntatore e si MUOVE "other" nello heap (box puntera' a questo). Poi si mette il box nella lista
        gameobjects_list.push(Box::new(base_enemy_2));

        let enemy_spawn_rate_range = (4.0, 7.0);
        let enemy_health_range = (5, 10);
        let enemy_speed_range = (20.0, 25.0);

        Ok(
            Game { 
                canvas: canvas_main, 
                event_pump: event_pump_main,
                resource_manager: resources,
                player: player,
                gameobjects: gameobjects_list,
                utils: Utils::new(),
                main_camera:Camera::new(),
                enemy_spawner:EnemySpawner::new(enemy_spawn_rate_range, enemy_health_range, enemy_speed_range),
                game_score:0,
                last_score_checkpoint:0,
            }
        )
    }

    pub fn start(&mut self) -> Result<(), String>{ // Inizializzazione oggetti di base (come player ecc...)
        
        // da fare poi in un eventuale file ResourceManager
        // da errore in quanto in questo modo la texture vive meno di Game (in quanto viene scartata alla fine di start)
        // mentre la texture poi deve vivere quanto il player (che essendo poi dichiarato in Game vive quanto esso)

        // self.gameobjects.push(Box::new(player));

        Ok(())
    }

    pub fn manage_events(&mut self) -> bool {
        
        for event in self.event_pump.poll_iter(){
            self.player.move_player(&event);
            self.player.player_controller(&event, &mut self.gameobjects);

            self.utils.utils_manage_events(&event);

            match event{
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} =>{
                    return false;
                }

                _ => {
                    continue;
                }
            }
        }

        return true;
    }

    pub fn render(&mut self) -> Result<(), String>{
        self.canvas.set_draw_color(Color::RGB(0, 0, 0)); // colore di sfondo
        self.canvas.clear(); // si imposta colore scelto

        // rendering player
        // utilizzo game_utils in modo da portare tutti gli oggetti nel S.R della camera
        self.player.draw(self.canvas, self.resource_manager
            .get_texture("player").unwrap(), 0, &self.utils, 1.0).expect("Errore renderizzando");


        // rendering vari gameobjects
        if self.gameobjects.len() > 0{
            for game_object in self.gameobjects.iter_mut(){
                if let Some(bullet) = game_object.as_any_mut().downcast_mut::<Bullet>(){
                    let bullet_texture_scale_factor = 0.2;

                    bullet.draw(self.canvas, self.resource_manager.get_texture("bullet").unwrap(), 
                        0, &self.utils, bullet_texture_scale_factor).expect("Errore rendering bullet");
                }else{
                    game_object.draw(self.canvas, self.resource_manager
                        .get_texture("default").unwrap(), 0, &self.utils, 1.0).expect("Errore renderizzando");
                }
            }
        }

        // FONTS
        // creo il ttf context. utilizzo map_err() in modo che, se da errore, invece di restituire
        // Result<ttfContext, Error> restituisco Result<ttfContext, String>, questo permette di convertire
        // l'errore in stringa facilmente. SI utilizza una funzione lambda, dove "e" rappresenta
        // il tipo Error e io lo converto direttamente in stringa con e.to_string()
        // ttf_context e' l'oggetto per gestire i font
        let ttf_context = sdl2::ttf::init().map_err(|e| {e.to_string()})?;
        // Path::new() restituisce &Path in quanto serve dimensione nota
        let font_path:&Path = Path::new("fonts/Roboto_Condensed-Black.ttf");
        let mut font = ttf_context.load_font(font_path, 128)?;
        font.set_style(sdl2::ttf::FontStyle::NORMAL);

        // stampa delle varie cose che devono essere stampate
        // N.B => Il testo rimane fisso nello schermo in quanto a spostarsi nella direzione opposta della camera
        // sono solo le entities (vedi impl Gameobject for Entities, qui nell'update si spostano le entities rispetto alla camera)

        // stampa vita player:
        let player_health_pos = Point::new(10, 10);
        let player_health_size = Point::new(120, 40);
        Utils::write_on_screen(format!("health: {}", self.player.get_current_health()).as_str(), self.canvas, &font, &self.resource_manager,
        player_health_pos, player_health_size)?;

        // stampa game score

        // prendo width ed height del canvas attuale
        let score_pos = Point::new((self.canvas.output_size()?.0 - 150) as i32, 10 as i32);
        let score_size = Point::new(120, 40);
        Utils::write_on_screen(format!("Score : {}", self.game_score).as_str(), self.canvas, &font, &self.resource_manager, score_pos, score_size)?;

        if self.player.is_destroyed(){
            let (canvas_width, canvas_height) = self.canvas.output_size()?;
            let text = "GAME OVER";

            let text_size = Point::new(300, 80);
            let text_pos = Point::new(
                (canvas_width / 2) as i32 - (text_size.x / 2),
                (canvas_height / 2) as i32 - (text_size.y / 2),
            );
        
            Utils::write_on_screen(
                text,
                self.canvas,
                &font,
                &self.resource_manager,
                text_pos,
                text_size,
            )?;
        }

        self.canvas.present(); // si renderizza canvas
        Ok(())
    }

    pub fn update(&mut self, deltatime:f32){

        if self.player.is_destroyed(){
            return;
        }

        // non posso passare come parametro &game in quanto avrei in contemporanea un riferimento mutabile (&mut self)
        // e uno immutabile (quello che voglio passare come parametro ad update)

        // SETTAGGIO UTILS
        self.utils.save_player_position(self.player.player_entity.get_position()); //Salvo in utils la posizione del player
        self.utils.main_camera_position = self.main_camera.get_main_camera_position(); // salvo in utils la posizione della camera
        // in modo da poterla usare negli update dei vari gameobjects

        // CAMERA : Spostamento in base a posizione del player
        self.main_camera.update(deltatime, &self.utils); // qui salbo la posizione della camera

        // UPDATE DEI GAMEOBEJCTS

        self.player.update(deltatime, &self.utils);


        // eseguire l'update di tutti gli altri gameobjects
        for game_object in self.gameobjects.iter_mut(){
            game_object.update(deltatime, &self.utils);

            // nemici che attano player al contatto
            if let Some(enemy) = game_object.as_any_mut().downcast_mut::<Enemy>(){
                enemy.damage_player(&mut self.player);
            }
        }

        // questo metodo per rimuovere bullet da Vec non funziona in rust in quanto remove() e' mut ref
        // quindi non posso avere altri riferimenti (anche se non mutabili) a gameobjects insieme
        // for (index, game_object) in self.gameobjects.iter_mut().enumerate(){
        //     if let Some(bullet) = game_object.as_any().downcast_ref::<Bullet>(){
        //         self.gameobjects.remove(index);
        //     }
        // }
        // quello che posso fare e' usare il metodo retain:
        self.gameobjects.retain(|game_object| {
            // vecchie implementazioni : 

            // tengo solo i gameObject che non sono destroyed
            // !game_object.is_destroyed()
            // in questo modo vengono droppati dalla lista gameobjects, quindi non eseguono piu' update e rendering! (si dealloca lo spazio)

            // if let Some(bullet) = game_object.as_any().downcast_ref::<Bullet>(){ // per i gameobject che sono bullet
            //     !bullet.is_destroyed() // tieni il bullet solo se NON e' distrutto
            // }else{
            //     true // tengo tutti gli altri
            // }
            
            // implementazione che incrementa lo score:

            if game_object.is_destroyed(){ // se il gameobject e' destroyed
                if let Some(enemy) = game_object.as_any().downcast_ref::<Enemy>(){
                    self.game_score += 10; // se e' un enemy in particolare, prima di rimuoverlo incremento score
                }
                false // ritorno false quindi lo tolgo dalla lista
            }else{
                true // altrimenti lo tengo nella lista
            }
        });

        // println!("{}", self.gameobjects.len());

        // SPAWN ENEMIES
        // qui solo uno alla volta
        // let new_enemy = self.enemy_spawner.spawn_enemy(deltatime, &self.utils);
        // match new_enemy {
        //     Some(new_enemy) => { // se ho effettivamente spawnato enemy, lo metto nella lista
        //         self.gameobjects.push(Box::new(new_enemy));
        //     },
        //     None =>{ // se None non si fa niente (ovvero il tempo sta ancora scorrendo)
// 
        //     }
        // }

        // metodo che permette di spawnare piu' nemici alla volta
        let new_enemies = self.enemy_spawner.spawn_enemies(deltatime, &self.utils);
        match new_enemies {
            Some(new_enemies) => {
                for enemy in new_enemies{ // voglio che vengano consumati, quindi senza iter
                    self.gameobjects.push(Box::new(enemy));
                }
            },
            None => {

            }
        }

        // COLLISIONE BULLETS

        // per collisione bullet
        // devo creare due liste separte perche non posso fare due cicli annidati entrambi con borrow mutabile su gameobjects
        // quindi metto bullet ed enemies in un vec e poi ciclo su questi dopo

        // N.B per ora non sto sfruttando a pieno il metodo generico bullet.damage_enemy!
        // spostare eventualmente queste liste come metodi generali di Game.rs
        let mut bullets = vec![];
        let mut enemies = vec![];

        for obj in self.gameobjects.iter_mut() {
            // utilizzo il match in modo da evitare di avere due as_any_mut() nello stesso ciclo
            // questo causa problemi con il borrow checker in quanto avrei 2 riferimenti mutabili &mut dyn Any
            // in questo modo ne ho solo uno e metto nella lista sulla base della struct che implementa il tratto gameobject
            match obj.as_any_mut() {
                any if any.is::<Bullet>() => {
                    let bullet = any.downcast_mut::<Bullet>().unwrap();
                    bullets.push(bullet);
                },
                any if any.is::<Enemy>() => {
                    let enemy = any.downcast_mut::<Enemy>().unwrap();
                    enemies.push(enemy);
                },
                _ => {}
            }
        }

        for bullet in &mut bullets {
            for enemy in &mut enemies {
                bullet.damage_enemy(*enemy);
            }
        }

        if self.game_score >= self.last_score_checkpoint + 50 {
            self.enemy_spawner.increase_difficulty();
            self.last_score_checkpoint = self.game_score;
        }

    }
}