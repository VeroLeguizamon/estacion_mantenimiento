// Mecánico durmiendo (no hay Robots) -> Robot lo despierta y es reparado.
// Mecánico ocupado, bahías libres -> Robot ocupa una bahía.
// Mecánico ocupado y bahías no disponibles -> Robot vuelve a trabajar.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use rand::Rng;

const BAHIAS_ESPERA: usize = 4;
const TOTAL_ROBOTS: usize = 10;

struct Estado {
    esperando: VecDeque<usize>, 
    mecanico_libre: bool,
    robots_restantes: usize,
}

impl Estado{
    pub fn crear() -> Self{
        Self{
            esperando: VecDeque::new(),
            mecanico_libre: true,
            robots_restantes: TOTAL_ROBOTS,
        }
    }
}

struct EstacionMantenimiento{
    estado: Mutex<Estado>,
    hay_robots: Condvar,
}

impl EstacionMantenimiento{
    pub fn crear() -> Self{
        Self {
            estado: Mutex::new(Estado::crear()),
            hay_robots: Condvar::new(),
        }
    }

    // Simula el arreglo de un robot.
    pub fn arreglar_robot(&self) {
        let mut rng = rand::thread_rng();
        let duracion = rng.gen_range(200..500);
        thread::sleep(Duration::from_millis(duracion));
    }

}

fn ejecutar_mecanico(estacion: Arc<EstacionMantenimiento>){
    loop {
        let mut state = estacion.estado.lock().unwrap();

        // Espera mientras no entren robots.
        // Si ya no hay robots por atender, cierra la estación.
        // Si hay robots por atender, pero no llegaron a la estación se duerme.
        while state.esperando.is_empty() {
            if state.robots_restantes == 0 {
                println!("Mecánico: no quedan más clientes, estación cerrada.");
                return;
            }
            println!("Mecánico: no hay clientes, me duermo...");
            state = estacion.hay_robots.wait(state).unwrap();
        }

        // Llega un robot para arreglarse
        let robot_id = state.esperando.pop_front().unwrap();
        state.mecanico_libre = false;

        println!("Mecánico atendiendo al robot {}. Esperando ({})", robot_id, state.esperando.len());

        // Libera el recurso porque no lo necesita para arreglar el robot
        drop(state);

        estacion.arreglar_robot();

        // Libera al robot y se coloca como libre.
        let mut state = estacion.estado.lock().unwrap();
        state.mecanico_libre = true;
        state.robots_restantes -= 1;

        println!("Mecánico: terminé con el robot {} (restantes {})", robot_id, state.robots_restantes);
        drop(state);
    }
}

fn ejecutar_robot(estacion: Arc<EstacionMantenimiento>, id: usize){

    // Simula una llegada random a la estacion.
    let mut rng = rand::thread_rng();
    thread::sleep(Duration::from_millis(rng.gen_range(50..600)));    

    // El robot llega a la estación.
    let mut state = estacion.estado.lock().unwrap();
    println!("Robot {} llegó a la estación.", id);

    // Si no hay bahías de espera disponibles, el robot se va.
    if state.esperando.len() >= BAHIAS_ESPERA {
        println!("Robot {}: no hay bahía disponible, adiós", id);
        state.robots_restantes -= 1;
        estacion.hay_robots.notify_one();
        return;
    }

    // En el caso de que existan bahías disponibles, el robot se queda a esperar.
    state.esperando.push_back(id);
    println!("Robot {}: me siento a esperar (ocupadas: {}/{})", id, state.esperando.len(), BAHIAS_ESPERA);

    // Notifica al mecánico por si esta dormido.
    estacion.hay_robots.notify_one();
    drop(state);
}

fn iniciar_mecanico(estacion: Arc<EstacionMantenimiento>) -> thread::JoinHandle<()> {
    thread::spawn(move || ejecutar_mecanico(estacion))
}

fn iniciar_robot(estacion: Arc<EstacionMantenimiento>, id: usize) -> thread::JoinHandle<()> {
    thread::spawn(move || ejecutar_robot(estacion, id))
}

fn iniciar(){
    let estacion = Arc::new(EstacionMantenimiento::crear());

    let mecanico = iniciar_mecanico(Arc::clone(&estacion));

    let mut robots = vec![];
    for id in 1..=TOTAL_ROBOTS{
        robots.push(iniciar_robot(Arc::clone(&estacion), id));
    }

    for robot in robots {
        robot.join().unwrap();
    }
    mecanico.join().unwrap();

    println!("Estación de mantenimiento cerrado. Fin.")
}

fn main() {
    //TODO: se supone que los robots deberían realizar la recepción de los paquetes y aleatoriamente ir a la zona de 
    //mantenimiento para después volver a recepcionar paquetes
    
    let mut areas = vec![];
    //let mut areas = vec![thread::spawn(recepcion::iniciar)];
    areas.push(thread::spawn(iniciar));

    for area in areas {
        area.join().unwrap();
    }
}
