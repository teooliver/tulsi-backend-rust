# TODOs:

- Add observability (Grafana)
- Swagger documentation
- Add more robust migrations system
- Authentication/Authorization (This is more complex as it might be worth using a third party to do it, need to do more research)
- Keep history on Tasks and Project changes (Title, Description, Assignment, etc).

# CURL Tests

## Create Task

```bash
curl -X POST http://localhost:3000/tasks \
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
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "My new task"}'
```
