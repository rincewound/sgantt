== sÜper Gantt

Input:
- A general configuration, containing:
    - The regular working days
    - The regular "output" of a generic resource
    - public holidays
- A number of tasks, each with:
  - a given duration (in hours!)
  - Optional: A number of predecessors
  - Optional: A "start earliest" date
  - Optional: The number of generic resources planned
- A number of resources, each with a given "output" of "duration units" per calendar duration
- A number of resource assignments, whith:
    - A resource that is being assigned
    - A percentage, telling us how much of the resource's output is to be spent on the assignment
        - Does this make sense?
- A calendar, that tells us when a given resource has a different availablity (i.e. when the output is different)



Output:
- A Gantt Chart, showing how the tasks interact and how long they take in calendar time, assuming they are processed using generic resources
- A Gantt Chart, showing how the tasks interact and how long they take in calendar time, assuming they are processed *only* by the resources assigned to them. Further, the actual assigned workforce is to be displayed (i.e. if they were generic resources.)
- A chart showing the load of each resource over the course of time.


Project Format (JSON):
```json
{
    "general": {
        "working_days": ["mo", "tue", "wed", "thu", "fri"],
        "regular_output" : 40,      // Should probably be minutes instead of hours here!
    }
    "tasks": [
        {
            "id": 0,
            "duration": 420,       // Should probably be minutes instead of hours.
            "label": "Some task!",
            "planned_resources": 3.0, // i.e. 3.0 * "regular_output" 
            "earliest_start_date": "2023-06-08"
        },
        {
            "id": 1,
            "duration": 540,       // Should probably be minutes instead of hours.
            "label": "Another task",
            "planned_resources": 3.0, // i.e. 3.0 * "regular_output" 
            "earliest_start_date": "2023-06-08",
            "predecessors": [0]   // this will shift the startdate to "after the predecessor is done."
        }

    ]
    "resources": [
        {
            "id": 0,
            "label": "A Resource Name",
            "output": 32,       // i.e. an 80% worker
        }
    ]

    "assignments": [
        {
            "task": 0,
            "resource": 0,
            "percentage": 50    // in percent, this would mean 32 * 50%
        }
    ]
}
```

== Interpreting the output

=== Gantt Chart
This is your typical Gantt chart that displays which tasks are being processed when. However, there are
some additional informations:
- On the left, next to the task's name the "FTE" number displays the "planned_resources" value for this chart. The "days" number denotes the number of __working__ days of effort that remain on this task.
- On the right, next to each bar we can see the allocated resources and the percentage to which they work on this taks.
- The length of each bar is calculated by two means:
    - If no resources were assigned to the task, the length is calculated "as-if" resources were available according to the planned number.
    - If resources are assigned to the task, the length is calculated based on the actual number of resources available to the task.

=== Load Chart
The load chart will calculate the speculative load for all resources in the project for the next 365 days, based on the assignments.