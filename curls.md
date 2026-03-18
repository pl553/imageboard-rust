1. Аутентификация (Auth)

    Получение токена (Login)

    ```
    curl -X POST http://localhost:8080/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d '{
        "username": "admin",
        "password": "securepassword"
    }'
    ```

    Проверка текущего пользователя (Get Me)
    
    ```
    curl -X GET http://localhost:8080/api/v1/auth/me \
        -H "Authorization: Bearer YOUR_JWT_TOKEN_HERE"
    ```

    Смена пароля

    ```
    curl -X POST http://localhost:8080/api/v1/auth/change-password \
        -H "Authorization: Bearer YOUR_JWT_TOKEN_HERE" \
        -H "Content-Type: application/json" \
        -d '{
            "current_password": "securepassword",
            "new_password": "newsecurepassword123"
        }'
    ```

2. Управление досками (Boards)

    Список всех досок
    
    ```
    curl -X GET http://localhost:8080/api/v1/boards
    ```

    Создание новой доски (Admin only)

    ```
    curl -X POST http://localhost:8080/api/v1/boards \
        -H "Authorization: Bearer YOUR_JWT_TOKEN_HERE" \
        -H "Content-Type: application/json" \
        -d '{
            "slug": "test",
            "name": "Test Board",
            "description": "Board for testing API"
        }'
    ```

    Получение информации о доске

    ```
    curl -X GET http://localhost:8080/api/v1/boards/test
    ```

    Удаление доски (Admin only)

    ```
    curl -X DELETE http://localhost:8080/api/v1/boards/test \
        -H "Authorization: Bearer YOUR_JWT_TOKEN_HERE"
    ```

3. Треды (Threads)

    Список тредов на доске (с пагинацией и превью)

    ```
    curl -X GET "http://localhost:8080/api/v1/boards/b/threads?page=1&limit=10&preview_posts=3"
    ```

    Создание нового треда

    ```
    curl -X POST http://localhost:8080/api/v1/boards/b/threads \
        -F "name=Anonymous" \
        -F "text=Это первый пост в новом треде!" \
        -F "image=@/path/to/your/image.jpg"
    ```

    Получение полного треда со всеми постами

    ```
    curl -X GET http://localhost:8080/api/v1/boards/b/threads/123456789
    ```

    Удаление треда (Admin only)

    ```
    curl -X DELETE http://localhost:8080/api/v1/boards/b/threads/123456789 \
        -H "Authorization: Bearer YOUR_JWT_TOKEN_HERE"
    ```

4. Посты (Posts)

    Создание ответа в треде

    ```
    curl -X POST http://localhost:8080/api/v1/boards/b/threads/123456789/posts \
        -F "name=Anonymous" \
        -F "text=Ответ на тред." \
        -F "image=@/path/to/another/image.png"
    ```

    Удаление поста (Admin only)

    ```
    curl -X DELETE http://localhost:8080/api/v1/boards/b/posts/987654321 \
        -H "Authorization: Bearer YOUR_JWT_TOKEN_HERE"
    ```

5. Изображения (Images)

    Получение полного изображения

    ```
    curl -X GET http://localhost:8080/api/v1/images/filename.jpg \
        --output downloaded_image.jpg
    ```

    Получение миниатюры (thumbnail)

    ```
    curl -X GET http://localhost:8080/api/v1/images/thumb/filename.jpg \
        --output downloaded_thumb.jpg
    ```