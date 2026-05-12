#!/bin/bash

# Настройки
SOURCE_DIR="/home/max/Public/rweb/flatdb"
CHROOT_PATH="/media/srv"
REMOTE_SERVER="max@humorltu.lt"
REMOTE_DEST="/usr/local/bin/flatdb"
# Путь для комплешнов (папка доступна для записи пользователю max)
REMOTE_COMP_DEST="/usr/share/bash-completion/completions/flatdb"

# Обработка аргументов
MODE="release"
CARGO_FLAGS="--release"

case "$1" in
    dbg)
        MODE="debug"
        CARGO_FLAGS=""
        echo "--- Сборка в режиме DEBUG ---"
        ;;
    rel)
        MODE="release"
        CARGO_FLAGS="--release"
        echo "--- Сборка в режиме RELEASE ---"
        ;;
    *)
        echo "Использование: $0 {dbg|rel}"
        exit 1
        ;;
esac

# 1. Синхронизация исходников в chroot (исключая target)
echo ">>> Синхронизация файлов в chroot..."
sudo rsync -avz --delete --exclude 'target/' \
    "$SOURCE_DIR/" "$CHROOT_PATH/home/max/flatdb/"

# 2. Сборка внутри chroot от пользователя max
echo ">>> Запуск сборки в chroot..."
sudo chroot "$CHROOT_PATH" /bin/bash -c "cd /home/max/flatdb && su max -c '
    source /etc/profile;
    export CC=clang;
    export CXX=clang++;
    export AR=llvm-ar;
    export NM=llvm-nm;
    RUSTFLAGS=\"-C linker=clang\" cargo build $CARGO_FLAGS'"

if [ $? -ne 0 ]; then
    echo "Ошибка: Сборка не удалась!"
    exit 1
fi

# 3. Пути к файлам
BINARY_PATH="$CHROOT_PATH/home/max/flatdb/target/$MODE/flatdb"
BUILD_OUT_DIR="$CHROOT_PATH/home/max/flatdb/target/$MODE/build"

# 4. Поиск самого свежего файла completion в активном режиме сборки
echo ">>> Поиск актуального bash completion..."
# Находим flatdb.bash, сортируем по времени изменения (%T@), берем первый
COMPLETION_SRC=$(find "$BUILD_OUT_DIR" -name "flatdb.bash" -printf "%T@ %p\n" 2>/dev/null | sort -nr | head -n 1 | cut -d' ' -f2-)

# 5. Загрузка бинарника на сервер
echo ">>> Загрузка бинарного файла на сервер $REMOTE_SERVER..."
scp "$BINARY_PATH" "$REMOTE_SERVER:$REMOTE_DEST"

# 6. Прямая загрузка completion на сервер
if [ -n "$COMPLETION_SRC" ] && [ -f "$COMPLETION_SRC" ]; then
    echo ">>> Найдено дополнение: $COMPLETION_SRC"
    echo ">>> Загрузка bash completion в $REMOTE_COMP_DEST..."
    # Копируем напрямую, так как права позволяют
    scp "$COMPLETION_SRC" "$REMOTE_SERVER:$REMOTE_COMP_DEST"
else
    echo "!!! Предупреждение: Файл completion не найден в $BUILD_OUT_DIR"
fi

if [ $? -eq 0 ]; then
    echo "--- Все операции успешно завершены! ---"
else
    echo "Ошибка при деплое."
    exit 1
fi
