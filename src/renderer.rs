use std::collections::HashMap;

use svg::{Document, node::{element::{path::Data, Path, self}}, Node};
use time::{Date};
use chrono::{prelude::*, Months};

use crate::pom::Project;

const BAR_START_X: u32 = 150;
const BAR_START_Y: u32 = 20;
const BAR_HEIGHT: u32 = 8;

struct Point
{
    pub x: u32,
    pub y: u32
}

struct TaskPoints
{
    pub start: Point,
    pub end: Point
}

fn add_v_line(doc: Document, x: u32) -> Document
{            
    let data = Data::new()
    .move_to((x, 0))    
    .line_by((0, 5000))
    .close();

    let path = Path::new()                    
    .set("fill", "none")
    .set("stroke", "black")
    .set("stroke-width", "1")
    .set("d", data);

    return doc.add(path);
}

fn add_top_offset_v_line(doc: Document, x: u32, yoff: u32) -> Document
{            
    let data = Data::new()
    .move_to((x, yoff))    
    .line_by((0, 5000))
    .close();

    let path = Path::new()                    
    .set("fill", "none")
    .set("stroke", "black")
    .set("stroke-width", "1")
    .set("d", data);

    return doc.add(path);
}

fn add_h_line(doc: Document, y: u32) -> Document
{            
    let data = Data::new()
    .move_to((0, y))    
    .line_by((5000, 0))
    .close();

    let path = Path::new()                    
    .set("fill", "none")
    .set("stroke", "black")
    .set("stroke-width", "1")
    .set("d", data);

    return doc.add(path);
}

pub fn add_text_at(doc: Document, text: &str, x: u32, y: u32) -> Document
{            
    let txt = svg::node::Text::new(text);            
    let mut text_elem = element::Text::new()
        .add(txt);
    text_elem.assign("x", x);
    text_elem.assign("y", y);     // weird magic to get the labels positioned correctly
    text_elem.assign("font-size", 8);
    text_elem.assign("fill", "black");    
    doc.add(text_elem)
}

fn date_to_chrono_naive(date: &Date) -> NaiveDate
{
    NaiveDate::from_ymd_opt(date.year(), date.month() as u32, date.day() as u32).unwrap()
}

pub fn start_of_quarter(date: NaiveDate) -> NaiveDate
{
    let month = date.month() as u32;
     // !!! Since January is not Month 0 we need to subtract 1 to get the correct result here
    let months_since_q_start = (month - 1) % 3;    
    let q_start = month -months_since_q_start;
    NaiveDate::from_ymd_opt(date.year(), q_start, 1).unwrap() 
}

pub fn next_quarter(date: NaiveDate) -> NaiveDate
{
    let start_of_this_q = start_of_quarter(date);
    start_of_this_q.checked_add_months(Months::new(3)).unwrap()
}

fn render_gantt_layout(start_date: Date) -> Document
{
    let mut document = Document::new();
    document = add_v_line(document, BAR_START_X);
    document = add_text_at(document, "Task", 0, 8);

    // add quarter lines:
    let start_date_chrono = date_to_chrono_naive(&start_date);
    let next_quarter_start = next_quarter(start_date_chrono);

    // render lines for the next 4 quarters:
    let mut quarter_start= next_quarter_start.clone();
    for _ in 0..=4
    {        
        let days = (quarter_start - start_date_chrono).num_days();
        document = add_v_line(document, BAR_START_X + days as u32);
        let label_text = format!("{}", quarter_start);
        document = add_text_at(document, &label_text, BAR_START_X + days as u32, 10);
        quarter_start = quarter_start.checked_add_months(chrono::Months::new(3)).unwrap();
    }
    document = document.set("style","background-color:white");
    document
}

pub fn date_to_x_pos(start_date: Date, rel_date: Date) -> u32
{
    if rel_date > start_date
    {
        return BAR_START_X + (rel_date - start_date).whole_days() as u32
    }
    BAR_START_X
}

pub fn render_gantt(p: &Project, start_date: Date) -> Document
{
    let mut document = render_gantt_layout(start_date);
    let mut task_number = 0;

    // We use this map to store the "end positions" for all tasks.
    // These are then used in a second pass to draw dependency arrows
    let mut task_start_and_end_points = HashMap::<u32, TaskPoints>::new();


    for task in p.tasks.iter()
    {        

        let task_start_date = task.get_actual_start_date(&p);
        let task_end_date = task.get_actual_end_date(&p);
        let days = task.get_actual_remaining_calender_days(&p, start_date.clone()) as i32;
        let working_days = task.get_actual_remaining_work_days(&p, start_date.clone()) as i32;


        if days > 0
        {
            let element_y = BAR_START_Y + BAR_HEIGHT * task_number + 2 * task_number;
            let element_x = date_to_x_pos(start_date, task_start_date);

            println!("Rendering {}, start date {} end date {}", task.id, task_start_date, task_end_date);

            let data = Data::new()
                            .move_to((element_x as u32, element_y))
                            .line_by((days, 0))
                            .line_by((0, BAR_HEIGHT))
                            .line_by((-days, 0))
                            .close();

            let path = Path::new()                    
                        .set("fill", "#A0A0CC")
                        .set("stroke", "#7979CC")
                        .set("stroke-width", "1")
                        .set("d", data);

            let task_label = format!("{}, {} days, {} FTE", task.label, working_days, task.planned_resources);
            document = document.add(path);                                           
            document = add_text_at(document, &task_label, 0, element_y + 6);
            document = add_h_line(document, element_y - 1 );

            
            task_start_and_end_points.insert(task.id, TaskPoints {
                start: Point{x: element_x as u32, y: element_y},
                end: Point{x: element_x + days as u32, y: element_y}
            });


            task_number += 1;
        }
    }

    document = render_dependency_arrows(&p, &task_start_and_end_points, document);    
    document = render_resources(&p, &task_start_and_end_points, document);    

    document
}

fn render_resources(p: &Project, task_start_and_end_points: &HashMap<u32, TaskPoints>, document: element::SVG) -> element::SVG {
    let mut the_doc = document;
    for task in p.tasks.iter()
    {
        let own_points = task_start_and_end_points.get(&task.id).unwrap();
        
        // find own allocations and create label for them:
        let mut label = String::from("");
        let mut sum = 0.0;
        for alloc in p.allocations.iter().filter(|x| x.taskid ==task.id)
        {
            let res = p.get_resource_by_id(alloc.resourceid);
            label.push_str(&format!("{}:{}%, ", res.label, alloc.load * 100.0));
            sum += alloc.load;
        }

        label.push_str(&format!("FTE:{}/{}", sum, task.planned_resources));

        the_doc = add_text_at(the_doc, &label, own_points.end.x, own_points.end.y + 6);
    }
    the_doc
}

fn render_dependency_arrows(p: &Project, task_start_and_end_points: &HashMap<u32, TaskPoints>, document: Document) -> Document
{
    let mut the_doc = document;
    // second pass, draw dependency arrows
    for task in p.tasks.iter()
    {
        let own_points = task_start_and_end_points.get(&task.id).unwrap();
        for pred_id in task.predecessors.iter()
        {
            let pred_points = task_start_and_end_points.get(pred_id).unwrap();

            let data = Data::new()
            .move_to((pred_points.end.x, pred_points.end.y + 4))
            .line_to((own_points.start.x, own_points.start.y + 4))
            .close();

            let path = Path::new()
            .set("fill", "none")
            .set("stroke", "blue")
            .set("stroke-width", "1")
            .set("d", data);
            the_doc = the_doc.add(path);                                           
        }
    }
    the_doc
}

pub fn render_resource_load_chart(p: Project, start_date: Date) -> Document
{
    let mut document = render_gantt_layout(start_date);

    document
}

#[cfg(test)]
mod tests
{
    use chrono::NaiveDate;

    use crate::renderer::next_quarter;

    use super::start_of_quarter;


    #[test]
    pub fn get_correct_start_of_quarter()
    {
        let d = NaiveDate::from_ymd_opt(2023, 6, 8).unwrap();
        let start_of_quarter = start_of_quarter(d);

        assert_eq!(
            start_of_quarter,
            NaiveDate::from_ymd_opt(2023, 4, 1).unwrap()
        )        
    }

    #[test]
    pub fn get_correct_start_of_quarter_jan()
    {
        let d = NaiveDate::from_ymd_opt(2023, 1, 8).unwrap();
        let start_of_quarter = start_of_quarter(d);

        assert_eq!(
            start_of_quarter,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()
        )        
    }

    #[test]
    pub fn get_correct_start_of_next_quarter()
    {
        let d = NaiveDate::from_ymd_opt(2023, 6, 30).unwrap();
        let start_of_quarter = next_quarter(d);

        assert_eq!(
            start_of_quarter,
            NaiveDate::from_ymd_opt(2023, 7, 1).unwrap()
        )        
    }

}