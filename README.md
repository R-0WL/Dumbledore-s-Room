# Despacho de Dumbledore - diorama de cubos con ray tracing (Rust)

*edit[30/10/2026]: Los repos estaban privados por que yo uso github como drive y tengo un script para crear repos donde uno solo pone repo <nombre> y te lo crea, yo solo copiaba el link generado en la tarea sin verificar como se veía después en la entrega [que cuando uno le da click al link uno si lo mira porque uno está autenticado en github web y me confiaba en eso]. Se me olvidó que el script es para repos privados y es el que uso normalmente, ya cree otro llamado repoclase que va a ser público y lo usaré en todos los repos de la U. Me disculpo por las molestias generadas :(*


## Ejecutar
```
cargo run --release
```
(En debug es muy lento. El proyecto usa solo `minifb` y `nalgebra-glm`, sin archivos externos.)

## Controles
| Tecla / raton | Accion |
|---|---|
| Flechas / A D / arrastrar con el raton | Rotar el diorama (yaw / elevacion) |
| W S / rueda del raton | Acercar / alejar |
| Tab | Cambiar manualmente entre escena exterior e interior |
| Espacio | Pausar el reloj del dia |
| `,` `.` (mantener) | Retroceder / adelantar el tiempo |
| 1..8 | Saltar a: amanecer, dia, mediodia, nublado, lluvia, atardecer, noche, medianoche |
| Z | Calidad de render (1/2, 1/3, 1/4, 1/1). Si va lento, baja la calidad |
| Esc | Salir |

## Las dos escenas
- **Exterior**: la torre del despacho flota sobre un mar de nubes; Harry en Buckbeak vuela alrededor.
  Al acercarte (W / rueda) a una ventana, la camara entra sola al **interior** (con un fundido).
- **Interior**: el despacho con detalle (escalera, libreros, retratos, chimenea, escritorio, pensadero,
  candelabro, Dumbledore y Fawkes). Si te alejas con la camara frente a una ventana, sales al exterior.

## Rubrica
- **Complejidad / atractivo**: torre circular, isla flotante con arboles y rocas, escalera, ~250 retratos/libros, 3 ventanas arqueadas, 3 modelos SDF animados.
- **Rotacion y zoom**: camara orbital en ambas escenas.
- **Materiales (17, cada uno con textura propia + albedo/especular/reflectividad/transparencia)**: ver `src/material.rs`.
- **Refraccion**: vitrales (n=1.5), liquido del pensadero (n=1.33) y dos esferas de cristal (Snell + Fresnel, entrada y salida).
- **Reflexion**: oro, plata, tarima barnizada, pizarra mojada por la lluvia, vitrales.
- **Skybox**: `src/sky.rs`, procedural y animado: ciclo continuo amanecer -> dia -> mediodia -> nublado -> lluvia -> atardecer -> noche estrellada (sol, luna, estrellas, nubes, lluvia). El clima varia por dia: no siempre llueve.
- **Fawkes**: material emisivo (brilla por si mismo) + luz puntual naranja que proyecta sombras.

## Estructura
`main.rs` (ventana, camaras, reloj) · `room.rs` (diorama) · `tracer.rs` (ray tracer) · `voxel.rs` (cubos + DDA)
`sdf.rs` + `model_data.rs` (modelos de tu zip) · `sky.rs` · `texture.rs` · `material.rs` · `camera.rs` · `util.rs`


