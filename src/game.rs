use sdl2::{image::LoadTexture, keyboard::Keycode, render::{Canvas, TextureCreator, WindowCanvas}, video::WindowContext, EventPump, Sdl};
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::pixels::Color;

use crate::modules::{Camera, Entity, EntityType, GameObject, Player, ResourceManager, Utils, Bullet};

pub struct Game<'l>{
    canvas: &'l mut WindowCanvas,
    event_pump: &'l mut EventPump,
    resource_manager: ResourceManager<'l>,
    player: Player,
    gameobjects:Vec<Box<dyn GameObject>>,
    main_camera:Camera,

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

        let mut player = Player::new("Player", 50.0);
        player.player_entity.set_sprite(51, 43);

        // entity di prova per testare il rendering
        let mut other:Entity = Entity::new("test_entity", EntityType::Other);
        other.set_sprite(51, 43);

        let mut gameobjects_list:Vec<Box<dyn GameObject>> = Vec::new(); // si crea la lista di gameobjects
        gameobjects_list.push(Box::new(other)); // si crea un puntatore e si MUOVE "other" nello heap (box puntera' a questo). Poi si mette il box nella lista

        Ok(
            Game { 
                canvas: canvas_main, 
                event_pump: event_pump_main,
                resource_manager: resources,
                player: player,
                gameobjects: gameobjects_list,
                utils: Utils::new(),
                main_camera:Camera::new(),
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

        self.canvas.present(); // si renderizza canvas
        Ok(())
    }

    pub fn update(&mut self, deltatime:f32){
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
            if let Some(bullet) = game_object.as_any().downcast_ref::<Bullet>(){ // per i gameobject che sono bullet
                !bullet.is_destroyed() // tieni il bullet solo se NON e' distrutto
            }else{
                true // tengo tutti gli altri
            }
        });

        println!("{}", self.gameobjects.len());
    }
}