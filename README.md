# YaBlog :: Yet Another Blog

В этом проекте создана полноценная система блога, которая объединяет следующие технологии: 
  * веб-сервер с HTTP и gRPC API, 
  * клиентскую библиотеку, 
  * CLI-инструмент, и 
  * WASM-фронтенд. 

Система включает регистрацию и аутентификацию пользователей через JWT-токены, что позволяет защитить операции создания, редактирования и удаления постов.

## пререквизиты:
* docker
* just
  
## референсы
- frontend: `http://localhost:8081`
- backend HTTP: `http://localhost:8080`
- backend gRPC: `localhost:50051`
- postgres: `localhost:5432`

## Запуск и тест
### 1. запустить БД postgres в докере 
```bash
docker run --rm --name yablog-postgres \
  -e POSTGRES_USER=user \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=blog_db \
  -p 5432:5432 \
  -d postgres:16-alpine
```

### 2. сервер/backend
* запустить сервер
```bash
just server
```
* в отдельном терминале создать первого пользователя
```bash
just registeruser;
```

### 3. тестирование cli клинта
* выполнить последовательность команд 
```bash
just loginuser;
just list;
just create1;
just list;
just get1;
just delete1;
just list;
just logoutuser;
# this one will fail - not authorized
just create1;
# this one will fail - no record
just get1;
# this one will fail - not authorized
just delete1;
just list;
``` 

### 4. web-приложение/frontend
* запуск 
```
cd blog-wasm; python3 -m http.server 8081
```
* тестрвоание в браузере `localhost:8081`

### 5. подчистка 
* выбрать терминал с запущенным ранее  server'ом (см п.#2) + Ctrl+C
* остановить(удалить контейнер с postgres)
```bash
docker stop yablog-postgres
```

