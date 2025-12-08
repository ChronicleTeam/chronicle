# Testing

## Test Strategy


### Backend

For the backend, our strategy was to have one test per function. The test should cover the success path along with every edge case within reason. The tests will not attempt to cover nested functions. The assumption here is that every function used by the tested function is covered in another test and is working as intended.

There are four main kinds of functions being tested:

- API Route Handler

These functions expose the API's functionality and handle validation, parsing, authentication, access control, and make calls to database functions. The tests try to cover every possible error and verify that successful requests are reflected in the database. It's important to test authentication and access control errors to ensure security.

- Database CRUD Functions.

These functions are responsible for interacting with the database at the lowest level using SQL queries. They perform no additional validation and are not expected to return errors. With that in mind, the strategy here is to only test the expected inputs, since unexpected input would be caught by route handlers. The goal is to verify that SQL queries run without errors and that the database contains the correct changes.

- Import/Export Functions

These functions are responsible for the conversion of .csv and .xlsx files and Chronicle's table data. These functions (particularly the import functions) are considered critical, as they directly touch a client's system and data. The strategy is then to test these functions with custom data such that it matches the expected result. Exported values cannot reliably be tested automatically once a final export has been made, as it relies on the re-import functions being flawless, which we cannot guarantee.    

- Utility Functions

These functions are utilities created to reduce code bloat and prevent repeated code. They are typically straightforward to test and involve covering a number of cases to verify that the logic is sound.

### Frontend
The frontend testing strategy is split into two categories: unit testing and browser testing.

- Unit testing

Unit testing was done with Vitest and mostly tested the API-calling functions found in `lib/api/`. The goal for unit testing was to test every API-calling function for correctness, which was mostly done except for a few outlying examples.

- Browser testing

Since the bulk of the frontend code is found within the Svelte components themselves, a more advanced testing method had to be used to these. Vitest Browser mode was used to test these components a in a real browser environment to ensure the tests correspond with user experience. The goal of the tests was to walk through common user paths for the different components. Due to difficulties in setting up this form of testing, component coverage is lower than expected.


## Test Results

### Backend

Successful tests: 112/112

| Function Coverage | Line Coverage      | Region Coverage    |
| ----------------- | ------------------ | ------------------ |
| 94.04% (505/537)  | 95.20% (4939/5188) | 87.53% (6179/7059) |

### Frontend

Successful tests: 108/108

| Function Coverage | Line Coverage      | Branch Coverage    |
| ----------------- | ------------------ | ------------------ |
| 69.13%            | 79.1%              | 88.6%              |
