# System Requirements
**Priorities**

| Level | Priority     |
| ----- | ------------ |
| 4     | Essential    |
| 3     | High         |
| 2     | Low          |
| 1     | Nice to have |


## Functional Requirements

### Data management
***
Chronicle must support creating, updating, and deleting user tables.

**Priority:** 4

***
Chronicle must support creating, updating, and deleting fields in user tables.

**Priority:** 4

***
Chronicle fields must allow the following data types:
 - Text
 - Integer
 - Decimal
 - Money
 - Progress
 - Date Time
 - Web Link
 - Email
 - Checkbox
 - Enumeration

**Priority:** 3

***
Chronicle must support creating, updating, and deleting entries in user tables.

**Priority:** 4

***
Chronicle shall be able to display data tables entries in a tabular format where the columns represent fields and the rows represent entries.

**Priority:** 3

***
Chronicle must be able to import data tables from CSV and Excel file formats.

**Priority:** 3

***
Chronicle must be able to export data tables to CSV and Excel file formats.

**Priority:** 3

### Data visualization
***
Chronicle must support creating, updating, and deleting dashboards.

**Priority:** 4

***
Chronicle must support creating, updating, and deleting charts for the specified dashboard.

**Priority:** 4

***
Chronicle dashboards must allow displaying all of its charts on a canvas

**Priority:** 3

***
Chronicle dashboards must allow placing charts on a canvas at different positions.

**Priority:** 2

***
Chronicle dashboards must allow changing the size of the charts on a canvas.

**Priority:** 2

***
Chronicle dashboards must be able to export the canvas to PNG and SVG file formats.

**Priority:** 1

***
Chronicle charts must be able to export to PNG and SVG file formats.

**Priority:** 1

***
Chronicle charts shall be able to display data in the following forms: Table, Bar Graph, Line graph 

**Priority:** 3

***
Chronicle charts may be able to display data in the following forms: Pie chart, Scatter plot 

**Priority:** 2

***
Chronicle must support setting the axes of the specified chart.

**Priority:** 4

***
Chronicle axes must refer to one specific data table field.

**Priority:** 4

***
Chronicle axes must be one of the following type: X axis, Y axis, Tooltip, Label 

**Priority:** 3

***
Chronicle axes must have the option to be one of the following aggregates: Sum, Average, Minimum, Maximum, Count 

**Priority:** 3

***
Chronicle charts shall allow the option to sort one or more axes by ascending or descending order.

**Priority:** 2

***
Chronicle charts shall allow the option to filter axes using the following field dependent criteria:
- Cells must match a defined pattern
- Cells must be within a value range

**Priority:** 2

### Access control
***
Chronicle data tables must be shareable with other users in the system.

**Priority:** 4

***
Chronicle shared data tables must have the following access rights: Owner, Editor, Viewer 

**Priority:** 3

***
Chronicle dashboards must be shareable with other users in the system.

**Priority:** 4

***
Chronicle shared dashboards must have the following access rights: Owner, Editor, Viewer 

**Priority:** 3

### User management
***
Chronicle must support creating, updating, and deleting users.

**Priority:** 4

***
Chronicle must only allow users with administrator access to manage users.

**Priority:** 3


## Non-functional Requirements
***
The system shall be accessible through a browser-based web application

**Priority:** 4

***
The system shall only be accessible to autheticated users.

**Priority:** 3

***
The system shall limit the  disk space usage of individual users.

**Priority:** 2

