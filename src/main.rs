use time::macros::date;

mod pom;
mod renderer;


fn main() 
{
    let mut the_project = pom::load_project("testinput.json");
    the_project.calculate_resource_allocations();
    let gantt = renderer::render_gantt(&the_project, date!(2023-06-08));    
    svg::save("image.svg", &gantt).unwrap();  
    let load_chart = renderer::render_resource_load_chart(the_project, date!(2023-06-08));
    svg::save("load_chart.svg", &load_chart).unwrap();  
}
