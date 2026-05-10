#!/bin/bash

# 1. Собираем проект
cargo build --release

# 2. Создаем папки, если их нет
mkdir -p ~/.config/srsq-panel

# 3. Копируем ассеты
cp resources/* ~/.config/srsq-panel/

# 4. Устанавливаем бинарник
sudo install -Dm755 target/release/srsq-panel /usr/local/bin/srsq-panel

echo "Установка завершена! Запускай через 'srsq-panel'."