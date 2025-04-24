use sdl2::{image::LoadTexture, keyboard::Keycode, render::{Canvas, TextureCreator, WindowCanvas}, video::WindowContext, EventPump, Sdl};
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::pixels::Color;

use crate::modules::{Entity, GameObject, Player, ResourceManager, EntityType, Utils};

pub struct Game<'l>{
    canvas: &'l mut WindowCanvas,
    event_pump: &'l mut EventPump,
    resource_manager: ResourceManager<'l>,
    player: Player,
    gameobjects:Vec<Box<dyn GameObject>>,

    utils:Utils,
}

impl<'l> Game<'l>{
    // ritorno result in quanto per creare canvas ecc necessito di propagare l'errore, vale comunque come costruttore
    pub fn new(canvas_main:&'l mut WindowCanvas, texture_creator:&'l TextureCreator<WindowContext>, event_pump_main:&'l mut EventPump) -> Result<Self, String>{
        let mut resources = ResourceManager::new(texture_creator);
        
        // qui carico tutte le textures
        resources.load_texture("player", "assets/survivor_sheet.png").expect("Errore caricamento textures");
        resources.load_texture("default", "assets/spritesheet_characters.png").expect("Errore caricamento textures");
        
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
            self.player.player_controller(&event);

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
        self.player.draw(self.canvas, self.resource_manager
            .get_texture("player").unwrap(), 0).expect("Errore renderizzando");


        // rendering vari gameobjects
        if self.gameobjects.len() > 0{
            for game_object in self.gameobjects.iter_mut(){
                game_object.draw(self.canvas, self.resource_manager
                    .get_texture("default").unwrap(), 0).expect("Errore renderizzando");
            }
        }

        self.canvas.present(); // si renderizza canvas
        Ok(())
    }

    pub fn update(&mut self, deltatime:f32){
        // non posso passare come parametro &game in quanto avrei in contemporanea un riferimento mutabile (&mut self)
        // e uno immutabile (quello che voglio passare come parametro ad update)
        self.player.update(deltatime, &self.utils);

        // eseguire l'update di tutti gli altri gameobjects
    }
}