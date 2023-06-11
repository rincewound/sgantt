
use std::{fs::File, collections::VecDeque};

use time::{Date, Duration, Weekday};
use serde::{Deserialize, Serialize};
use serde_json::{Value, Error};

/// THe PROJECT - OBJECT - MODEL
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Task
{
    pub id: u32,
    pub duration: u32,
    pub label: String,
    pub earliest_start_date: Date,
    pub planned_resources: f32,

    // These contain the actually allocated resources.
    #[serde(skip_deserializing)] 
    pub allocated_resources: f32,

    pub predecessors: Vec<u32>

    
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Resource
{
    pub id: u32,
    pub label: String,
    pub output: f32
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Allocation
{
    pub taskid: u32,
    pub resourceid: u32,
    pub load: f32
}


#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Project
{
    pub tasks: Vec<Task>,
    pub resources: Vec<Resource>,
    pub allocations: Vec<Allocation>
}

pub fn load_project(file_name: &str) -> Project
{
    let reader = File::open(file_name);
    if let Ok(file) = reader
    {
        let p: Project = serde_json::from_reader(file).unwrap();
        return p;
    }
    panic!("Not a valid project file!")
}


pub const GENERIC_RESOURCE_OUTPUT: f32 = 8.0;

impl Task
{
    /// Take a project and adjust the resources to the actual plan.
    pub fn set_allocated_resources(&mut self, allocated_resources: f32)
    {
        self.allocated_resources = allocated_resources;
    }

    fn get_work_force(&self) -> f32
    {
        if self.allocated_resources == 0.0
        {
            return self.planned_resources * GENERIC_RESOURCE_OUTPUT;
        }
        else 
        {
            return self.allocated_resources * GENERIC_RESOURCE_OUTPUT;
        }        
    }

    /// Returns the number of working days this task
    pub fn get_work_days(&self) -> f32
    {

        return self.duration as f32 / self.get_work_force();
    }

    /// Returns the end date given the assigned resources.
    pub fn get_end_date(&self) -> Date
    {
        let mut remaining_days = self.get_work_days();
        let mut current_date = self.earliest_start_date.clone();
        while remaining_days > 0.0
        {
            if let Some(_current_date) = current_date.checked_add(Duration::days(1))
            {
                current_date = _current_date;
            }
            else {
                print!("FAILED {}", current_date);
            }
            let wd = current_date.weekday();
            if ![Weekday::Saturday, Weekday::Sunday].contains(&wd)
            {
                remaining_days -= 1.0;
            }
        }
        current_date
    }

    // returns the number of calendar days between start and end
    pub fn get_calendar_days(&self) -> u32
    {
        let end = self.get_end_date();
        let dur = end - self.earliest_start_date;
        return dur.whole_days() as u32;
    }

    pub fn get_days_remaining_at(&self, reference_date: Date, start_date: Date) -> u32
    {
        let mut remaining_days = self.get_work_days();
        let mut current_date = start_date.clone();

        println!("RefDate {}, StartDate {}", reference_date, start_date);

        while remaining_days > 0.0 && current_date <= reference_date
        {
            current_date = current_date.checked_add(Duration::days(1)).unwrap();
            let wd = current_date.weekday();
            if ![Weekday::Saturday, Weekday::Sunday].contains(&wd)
            {
                remaining_days -= 1.0;
            }
        }
        //(remaining_days * (self.planned_resources * GENERIC_RESOURCE_OUTPUT)) as u32
        remaining_days as u32
    }

    // Returns the remaining "duration" at a given date:
    // pub fn get_remainder(&self, date: Date) -> u32
    // {
    //     self.get_days_remaining_at(date, self.earliest_start_date.clone())
    // }

    pub fn get_actual_end_date(&self, proj: &Project) -> Date
    {
        if self.predecessors.is_empty()
        {
            return self.get_end_date();
        }

        let actual_start_date = self.get_actual_start_date(proj);
        let calendar_days = self.get_calendar_days();
        let end_date =  actual_start_date.checked_add(Duration::days(calendar_days as i64)).unwrap();        
        end_date
    }

    pub fn get_actual_start_date(&self, proj: &Project) -> Date
    {
        if self.predecessors.is_empty()
        {
            return self.earliest_start_date;
        }

        // we have at least one predecessor        
        let mut the_start_date = self.earliest_start_date.clone();
        for pred_id in self.predecessors.iter()
        {
            if let Some(predecessor_task) = proj.tasks.iter().find(|x| x.id == *pred_id)
            {
                let pred_end_date = predecessor_task.get_actual_end_date(proj);
                if pred_end_date > the_start_date { the_start_date = pred_end_date ;}                
            }
        }        
        the_start_date
    }

    /// Calculates the actually remaining (duration!) days of the task relative to a given date,
    /// including the dependencies of the task.
    pub fn get_actual_remaining_work_days(&self, proj: &Project, reference_date: Date) -> u32
    {
        let start = self.get_actual_start_date(proj);
        self.get_days_remaining_at(reference_date, start)
    }

    /// Calculates the actually remaining calender days of the task relative to a given date,
    /// including the dependencies of the task.
    pub fn get_actual_remaining_calender_days(&self, proj: &Project, reference_date: Date) -> u32
    {
        let start = self.get_actual_start_date(proj);
        let end = self.get_actual_end_date(proj);

        if reference_date > end { return 0;}

        if start < reference_date && reference_date <= end
        {
            return (end - reference_date).whole_days() as u32 ;
        }
        else {
            if start > reference_date
            {
                return (end - start).whole_days() as u32;
            }
        }

        return 0;

    }

}


impl Project
{
    pub fn get_resource_by_id(&self, resource_id: u32) -> Resource
    {
        let res = self.resources.iter().find(|x| x.id == resource_id).expect(&format!("Need resource with correct id! {}", resource_id));
        res.clone()
    }

    pub fn get_resource_allocations_for_task(&self, task_id: u32) -> f32
    {
        let mut sum = 0.0;
        for alloc in self.allocations.iter().filter(|x| x.taskid == task_id)
        {                        
            sum += alloc.load;
        }
        sum
    }

    pub fn calculate_resource_allocations(&mut self)
    {
        let mut allocations: VecDeque<f32> = self.tasks.iter().map(|x| self.get_resource_allocations_for_task(x.id)).collect();
        for t in self.tasks.iter_mut()
        {            
            t.set_allocated_resources(allocations.pop_front().unwrap());
        }
    }

    pub fn calculate_resource_load(&self, resource_id: u32, day: Date) -> f32
    {
        let mut sum = 0.0;
        for relevant_allocation in self.allocations.iter().filter(|x| {x.resourceid == resource_id})
        {
            let task = self.tasks.iter().find(|x| {x.id == relevant_allocation.taskid}).expect("Resource Allocation does not apply to any existing task!");
            let start_date = task.get_actual_start_date(self);
            let end_date = task.get_actual_end_date(self);
            if day >= start_date && day <= end_date
            {
                sum += relevant_allocation.load;
            }
        }
        sum
    }

}

#[cfg(test)]
mod tests
{
    use super::{Task, Project, Resource, Allocation};
    use time::macros::date;
    use assertables;

    fn make_simple_task(duration: u32, planned_resources: f32) -> Task
    {
        Task {
            id: 0,
            duration,
            label: "".to_string(),
            earliest_start_date: date!(2023-02-01),
            planned_resources,
            predecessors: vec![],
            allocated_resources: 0.0,
        }
    }

    #[test]
    pub fn can_calculate_working_days()
    {
       let t = make_simple_task(80, 1.0);

       assert_eq!(10.0, t.get_work_days())
    }
    
    #[test]
    pub fn can_calculate_end_date()
    {
        let mut t = make_simple_task(80, 2.0);
        t.earliest_start_date = date!(2023-06-08);
        let end = t.get_end_date();
        
        // Should boil down to 5 workdays, however, since
        // 08/06 is a Thursday we have a weekend in between, which
        // puts the end to thursday the following week.
        assert_eq!(end, date!(2023-06-15))
    }

    // #[test]
    // pub fn can_get_remainder()
    // {
    //     let mut t = make_simple_task(80, 2.0);
    //     t.earliest_start_date = date!(2023-06-08);
    //     assert_eq!(64, t.get_remainder(date!(2023-06-09)))
    // }

    #[test]
    pub fn can_calculate_start_date_with_predecessor()
    {
        let mut proj = Project {
            resources: vec![],
            tasks: vec![],
            allocations: vec![],
        };

        let t0 = Task {
            id: 0,
            duration: 40,
            label: "First".to_string(),
            earliest_start_date: date!(2023-06-01),
            planned_resources: 1.0,
            predecessors: vec![],
            allocated_resources: 0.0,
        };

        
        let t1 = Task {
            id: 1,
            duration: 40,
            label: "Second".to_string(),
            earliest_start_date: date!(2023-06-01),
            planned_resources: 1.0,
            predecessors: vec![0],
            allocated_resources: 0.0,
        };

        proj.tasks = vec![t0,t1];

        let start_date = proj.tasks[1].get_actual_start_date(&proj);
        assert_eq!(start_date, date!(2023-06-08))

    }

    #[test]
    pub fn can_calculate_end_date_with_predecessor()
    {
        let mut proj = Project {
            resources: vec![],
            tasks: vec![],
            allocations: vec![],
        };

        let t0 = Task {
            id: 0,
            duration: 40,
            label: "First".to_string(),
            earliest_start_date: date!(2023-06-01),
            planned_resources: 1.0,
            predecessors: vec![],
            allocated_resources: 0.0,
        };

        
        let t1 = Task {
            id: 1,
            duration: 40,
            label: "Second".to_string(),
            earliest_start_date: date!(2023-06-01),
            planned_resources: 1.0,
            predecessors: vec![0],
            allocated_resources: 0.0,
        };

        proj.tasks = vec![t0,t1];

        let start_date = proj.tasks[1].get_actual_end_date(&proj);
        assert_eq!(start_date, date!(2023-06-15))

    }

    fn make_project() -> Project
    {
        
        let t0 = Task {
            id: 0,
            duration: 40,
            label: "First".to_string(),
            earliest_start_date: date!(2023-06-01),
            planned_resources: 1.0,
            predecessors: vec![],
            allocated_resources: 0.0,
        };

        
        let t1 = Task {
            id: 1,
            duration: 40,
            label: "Second".to_string(),
            earliest_start_date: date!(2023-06-03),
            planned_resources: 1.0,
            predecessors: vec![],
            allocated_resources: 0.0,
        };

        let proj = Project {
            resources: vec![],
            tasks: vec![t0,t1],
            allocations: vec![],
        };
        proj
    }

    #[test]
    pub fn can_calculate_resource_load_simple()
    {
        let mut project = make_project();
        let r = Resource {
            id: 0,
            label: "r1".to_string(),
            output: 40.0,
        };

        project.resources.push(r);

        let a = Allocation{ taskid: 0, resourceid: 0, load: 0.8 };
        let b = Allocation{ taskid: 1, resourceid: 0, load: 0.5 };
        project.allocations.push(a);
        project.allocations.push(b);

        let load = project.calculate_resource_load(0, date!(2023-06-01));
        assert_eq!(0.8, load);
        
        let load2 = project.calculate_resource_load(0, date!(2023-06-03));
        assert_eq!(1.3, load2);
    }

}
