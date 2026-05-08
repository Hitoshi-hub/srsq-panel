#!/bin/bash

# 1. Собираем проект
cargo build --release

# 2. Создаем папки, если их нет
mkdir -p ~/.config/srsq-panel
mkdir -p ~/.local/bin

# 3. Копируем ассеты
cp resources/* ~/.config/srsq-panel/

# 4. Копируем бинарник
cp target/release/srsq-panel /usr/local/bin/

echo "Установка завершена! Запускай через 'srsq-panel'."