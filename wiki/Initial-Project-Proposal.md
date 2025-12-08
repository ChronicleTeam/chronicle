# Chronicle

## Outline

An application for managing everyday data into collections and then easily create stylish dashboards for analysis. Chronicle can be used as a straightforward alternative to Excel for managing work or personal data without using a grid. Instead, it will use a graphical user interface (GUI) to create tables with typed fields, manage several collections, and filter/sort/search entries. A separate component of the app will allow users to create dashboards with graphs, aggregates, and sub-views. The GUI will focus on ease of use and avoid exposing the users to complex features. Essentially, Chronicle should be accessible to those without any experience with data science.

For example, someone could manage their bills, legal documents, groceries, family photos, and work hours, all in one place. Then, using beginner friendly tools, create a dashboard for bills spent per month, groceries count per week, photos and documents disk usage in gigabytes, and average work per day.

Chronicle will cover a wide range of use cases, for work, school, and personal life.

Finally, Chronicle will allow importing and exporting data and provide an option for local and cloud storage.

## Team Members and Roles

| Name           | Role                                                  |
| -------------- | ----------------------------------------------------- |
| Rémi Gros-Jean | Project Manager, Data visualization, Build management |
| Madeline Orr   | Architect, Back-end, Data management                  |
| Jeremy Dufour  | UI design, QA                                         |
| Baba Ly        | Front-end, Note-taker                                 |

## Customer

*Name*: Janelle McInnis

*Affiliation*: Defence Research and Development Canada (DRDC)

*Email*: janelle.mcinnis@forces.gc.ca

## Objectives

### Benefits to the customer
Our customer would use Chronicle to manage and share data with team members, getting a view of what is currently happening without having to ask for updates.

For example, Chronicle can be used to coordinate and manage different projects from team members. The client can create a table containing the project name, progress, blockers, members, and deadlines, then share it with team members. Team members can then modify progress, add blockers, or add notes for the client.

The client can then create a dashboard visualizing the progress over time of the members, see which projects are completed, and the most recent notes and blockers.

### Key things to accomplish
User interface and functionality for the following:

- Data management
  - Table creation, modification, and deletion.
  - Insert, modify, and delete table entries.
  - Create, modify, and delete table fields (field types TBD).
  - View tables and allow filtering and sorting by field.
  - Tables can be exported and imported (file formats TBD).
  - Tables can be shared among users with access control.

- Data visualization
  - Creation, modification, and deletion of dashboards
  - Dashboards allow for creating table views.
    - A table view is a selection of data from one or more tables that is possibly filtered, sorted, or aggregated.
  - Dashboards allow creating 2d plots (plot types TBD).
  - Dashboards allow placing the plots and views on a canvas.
  - Dashboards can be exported to an image file (file formats TBD).
  - Dashboards can be shared among users with access control.
  - Dashboards are updated when the data changes.


### Criteria for success
A functioning web application that can be opened on a modern web browser. The application should be accessible from a DRDC workstation and laptop. The application server and data should be hosted on a DRDC network for confidentiality.


## Expected Architecture

- Back end in Axum responsible for:
  - CRUD operations on tables
  - Data filtering, sorting, and aggregation
  - Data import and export
  - Generating plots
  - Sharing tables and dashboards
  - Table and dashboard access control

- Front end in Svelte for user interface and interactivity.

The web server and CDN should be hosted by DRDC.

## Anticipated Risks

- Difficulty integrating with DRDC networks.
- Difficulty implementing table and dashboard sharing.
- UI that is too complex.
- Features that are too complex/unnecessary.
- Too many features leading to complex code and design.
- Limited Rust knowledge and of its libraries for some members.
- Performance issues with data visualization.

## Legal and Social Issues

- Data confidentiality and security.
- Data loss prevention and integrity.
- User access control for sharing dashboards or tables.
- Created visualizations could be misleading.

## Initial Plans

Set up a local environment to run Axum and Svelte on one machine for development, with hot reloading. Look into containerization if its possible with DRDC servers. Focus on UI and basic functionality before import/export and data sharing.

