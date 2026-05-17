# Task Brief - Practical Task 5.2 (HD)

> Original task brief, preserved for reference. Not the working project todo (see `../TODO.md`) or plan (see `../PLAN.md`).

**Submission deadline:** 23:59 Sunday, last day of Week 12
**Demonstration deadline:** 23:59 Friday, last day of Week 13

## General Instructions

With everything you have learned so far, you should be able to implement your own backend service for a different domain rather than the one used in the unit. For this particular task, you will be given a domain, and you will implement several bounded contexts from the domain. You can also work in groups if you want to develop a more extensive working backend solution and have more bounded contexts covered at the end of the project.

## Learning Goals

You will learn to analyze the initial requirements and will go through the process of prototyping your own backend service. You will also learn how to research the domain, develop the project's ubiquitous language, and identify the bounded contexts to work on. In the end, you should have an implemented backend service for a specified domain.

## Technology Stack Requirements

You can use the ASP.NET Core Web API technology stack for Web API and the PostgreSQL database engine for the database. You are also encouraged to try some new features of the technology stack that were not covered by the unit's materials. You can use a completely different technology stack for the whole backend service or one of the layers of your application.

### What can be done differently to get maximum credit

- Use a different Web API tech stack (e.g. Express.js, Django)
- Use a different DB engine instead of PostgreSQL (e.g. MongoDB)
- Use a different ORM library (e.g. Dapper.NET)
- Implement a complex DB structure within PostgreSQL using indexes and PostgreSQL-specific functions
- Add a front-end to the backend service (e.g. VueJS, ReactJS)
- Build complex documentation (DB diagrams, system architecture diagrams, custom XML Documentation)
- Implement an Authentication Layer using uncovered approaches (e.g. OAuth, OIDC, Auth0)
- Use the PostGIS module of PostgreSQL to represent geographical data
- Implement a testing framework (e.g. xUnit, NUnit, MSTest)
- Build a complete Business Requirements Document with Aggregate Design Canvases
- Work in a group using git and a task management tool (GitHub, Jira, Trello)

## Domain

You need to build a backend service for the **art gallery of aboriginal art of Australia**. You don't need a fully working Web API that covers all possible usages for the art gallery. You can pick some of the possible bounded contexts to implement.

### Possible bounded contexts

- Artists
- Artifacts (any piece of art)
- Aboriginal Symbols (Iconography)
- Art Facts (10 facts about aboriginal art)
- Art Types
- Aboriginal Tribes
- Maps (manage geographical data about some other bounded context)
- Users
- Membership and Roles
- Art Gallery
- Art Eras (centuries / timeframes)
- Exhibitions
- Art Styles
- Comments (can be implemented along with Users)
- Tags (managing tags applied to other contexts)
- Your own bounded context not mentioned here

## Coding Requirements

You need to implement at least **3 bounded contexts** to complete this task. Please don't implement more than **5 bounded contexts** as it might complicate your solution and affect your ability to submit on time.

The final solution should be demonstrated to the teaching staff, and you need to show a complete understanding of the whole process of development. When working in groups, you need to explain your specific role and what was implemented by you.

## Reference Resources

- Kate Owen Gallery - Contemporary Aboriginal Art
- Aboriginal Art Code
- Harvard Art Museums API Documentation
- The Metropolitan Museum of Art API
- Art Institute of Chicago API
