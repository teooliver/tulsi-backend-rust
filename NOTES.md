# TODOs:

- Swagger documentation
- Authentication/Authorization (This is more complex as it might be worth using a third party to do it, need to do more research)
- Keep history on Tasks and Project changes (Title, Description, Assignment, etc).
- RoadMap Items (That can be linked to tasks or Projects)
- Workspaces (So we can organize projects, roadmaps, tasks and etc in those Workspaces)
- Add refresh token mechanism so we can make the jwt expire date a lot shorter and be able to "revoke" tokens

# CURL Tests

## Create Task

```bash
curl -X POST http://localhost:8080/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "My new task",
    "description": "Some description",
    "project_id": null,
    "assigned_to": null,
    "column_id": null
  }'
```

Minimal (only required fields):

```bash
curl -X POST http://localhost:8080/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "My new task"}'
```

# Run on any port:
PORT=9090 cargo run -p server
