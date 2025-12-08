# Project Summary

## Contributions


| Member         | Contributions                      | Time (22 weeks) |
| -------------- | ---------------------------------- | --------------- |
| Rémi Gros-Jean | Client liason, backend, deployment | 10 hours/week   |
| Madeline Orr   | Initial architecture, backend      | 10 hours/week   |
| Jeremy Dufour  | UI design, frontend                | 9.5 hours/week   |
| Baba Ly        | Frontend                           | 10 hours/week   |


## First semester

Here is a list of each issue we've encountered and how we fixed them:

### Challenges

**Child table design and implementation**

We had some initial trouble designing the data structures and database schema to allow users to create child tables. We initially thought about creating a recursive data structure where a field can refer to another table. We decided against it, and instead, each table has an optional parent ID. Child tables are identical to regular tables except that they contain a reference to an entry in the parent table. This made it so that almost no API changes were necessary.


**Charts and axis creation API**

Designing the chart and axis creation API was challenging because we wanted a generic API that applies to all different chart types. So, we took inspiration from Tableau software and decided to represent every kind of data visualization as a list of axes. Each axis refers to a field in a table and can optionally have an aggregate function. This made creating the dynamic SQL view for the chart simple. We also assign an axis type for each axis for the front-end to know how to build the chart (e.g. x and y axis, label, tooltip, etc).

**Field kind editing**

When implementing the Field kind editor, where users are able to change certain field parameters like the range of acceptable values, Chronicle needed to dynamically change the number and types of inputs depending on the type of field being edited. For example, an Integer field would need two number inputs to set a range, while the Text Field only needs a checkbox-type input to indicate whether it is required. To solve this, a custom component, VariableInput, was built. Alongside a new InputParameter type, it allowed Chronicle to change the inputs on display based on the current state of the fields.

**Field edit summary**

One goal for the field editor was to provide an overview of all changes made to the fields to the user before confirming their changes. Creating and maintaining such a list over the course of the editing session manually would have been a nightmare to debug, as users make and undo changes at will. Instead, Svelte's `$derived` was leveraged. When the field editor first starts up, it imports the table as it is stored in the backend. This table is subsequently copied, and this copy is what gets modified by the user during the editing session. `$derived` values are then used to determine the difference between the old table and the new table as the reactive states are updated. In this way, changes are tracked automatically and can then be converted to a human-readable format when necessary.

### Work completed

For the first semester, we have completed everything we planned to include in our MVP:
- Features
  - Data management
     - CRUD for tables, fields, and entries
     - Field types: text, integer, decimal, money, progress, datetime, web link, checkbox, ennumeration 
     - Field type conversion
     - Import/export for CSV and Excel
     - Sub-tables
  - Data visualization
     - CRUD for dashboards, charts, and axes
     - Chart types: bar, line, table
     - Axis aggregates: sum, average, min, max, count
     - Axis types: x, y, label, detail
     - Chart generation
  - User authentication
- Back-end and front-end remote deployment using managed services

Our plan for the second semester is to complete the following tasks:
- Back-end and front-end unit testing
- UI design review and user feedback
- Table and dashboard sharing, access control, and ownership transfer
- Allow users to move charts on a canvas grid in their dashboards
- User disk storage metrics
- Administrator page
  - Create/delete users
  - Setting disk usage limits
  - Managing user resources
- Integration and deployment for the client organization

## Second semester

### Challenges

**Testing dynamic SQL**

Chronicle's backend features dynamic SQL queries that change structure depending on user input. These queries were difficult to verify and were the greatest source of bugs. Additionally, these queries accommodate many different data types depending on field kinds and axis aggregates, which makes testing all combinations and edge cases unreasonable.

**Testing coverage**

Chronicle has a high flexibility in user inputs, which allows for many possible combinations. For instance, users may attempt operations with an invalid access role or may create tables with different combinations of fields. This necessitates testing many scenarios and edge cases, and results in a time-consuming effort to achieve satisfying coverage.

**Browser testing**

An important part of testing the frontend was to test in an automated browser component. This was done to test the frontend of the app using the same interface the end-user would have to use. The process to setup this testing environment turned out to be more difficult than expected. For example, the Playwright backend for Vitest Browser mode insisted on using its own versions of the test browsers to conduct the tests, which did not interact well with the operating system of some. One browser did not work nicely with the tests for unknown reasons, and much time was spent trying to find the root cause of this problem to no avail.

**Frontend URL Refactor** 

To better reflect a traditional website, the frontend was refactored to rely more on the URL to determine the frontend’s state (as opposed to storing state in a single component, which would have multiple views). Due to the new way data fetching was implemented to complement the refactor, there were some difficulties in finding the right way to refresh state. Time had to be spent looking more into Svelte documentation to achieve behaviour similar to the pre-refactor fronted. However, in the end, the refactor improved UX and organized the frontend code in a better way. 
### Work completed

For the second semester, we completed our main tasks and features:
- Features:
  - Table and dashboard sharing with access control
  - User management by admin users
- UI design enhancements
- Deployment on GCP
- CI/CD pipeline
- Near complete backend testing coverage
- Frontend testing coverage of critical features
- Backend OpenAPI documentation

Here's what we did not have the time or resources to do:
- Features:
  - Filtering table data
  - Sorting table columns
- Client on-site deployment
- Gather comprehensive user feedback

## Lessons Learned

We learned several lessons over the course of this project, both in terms of what worked well and what didn't.

Things that worked well include:
- Rust for backend/data-management.
- Formalized project management is beneficial.
- GCP was easy to use.
- Separate Git branches by feature and member.
- New skills and knowledge acquired.

For these, one of the first decisions we made for our backend architecture was to use Rust and its accompanying tooling, and that decision turned out to be one of the best choices we made. The type system allowed for seamless integration and quality of life with our database schemas, the surrounding tooling (such as cargo) accelerated development dramatically with easy package management, documentation, and testing, and Rust's design philosophy preemptively prevented many potential issues we might've encountered had we chosen a different language.

Things that didn't:
- Maintaining a consistent work schedule during the off-season.
- Part-time scheduling conflicts.
- Thorough testing, which now occupies more LOC than non-tests. 

By far the two largest conflicts encountered were the attempts to return to the project after the off-season, causing large amounts of delays as people re-acquainted themselves with the project's large codebase, and properly balancing priorities from other classes and life responsibilities with the project. A project of this scale does not lend itself well to part-time work, and especially not with a 4-month break in the middle. A better decision would have been to maintain a higher level of work output during the off-season.

While we do take pride in our rigorous test suite and acknowledge that the overwhelming majority of our written code was on critical functionality that laid the groundwork for this project should it go forward, we also must note that this took up an large amount of time and effort. 

Lastly, we believe it was extremely important that this project was used more as a critical chance for learning necessary skills for us going forward as engineers, and it taught us many valuable skills we can now use in the years to come.

## Final Achievements

### Backend

We have created a well-tested, organized, and documented Rust Axum API that exposes all the core functionality of Chronicle. Our backend implements logic to generate DDL SQL statements to store user data in a user-defined format.

### Frontend

We have designed a clean and minimalist UI that is intuitive and aesthetically appealing. Our frontend implements the logic necessary to dynamically generate tables and charts and is thoroughly tested.

### Deployment and CI/CD

We have used Terraform to build and maintain scalable Google Cloud Platform resources to deploy Chronicle in a production environment and manage CI/CD.

