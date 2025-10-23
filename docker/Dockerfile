# 选择一个基础镜像
FROM ubuntu:latest

# 设置工作目录
WORKDIR /app

# 将可执行文件、启动脚本和 libssl1.1 包复制到容器中
COPY powchain /app/powchain
COPY start.sh /app/start.sh
COPY libssl1.1_1.1.1f-1ubuntu2.20_amd64.deb /app/

# 安装必要的依赖
RUN apt-get update && apt-get install -y \
    wget \
    build-essential \
    libssl-dev \
    clang \
    libclang-dev \
    pkg-config \
    librocksdb-dev \
    gcc \
    libcurl4 \
    libcurl4-openssl-dev \
    openssl \
    libc6-dev \
    make \
    autoconf \
    automake \
    curl \
    git \
    ca-certificates \
    zlib1g-dev \
    libbz2-dev \
    liblzma-dev \
    libreadline-dev \
    libffi-dev \
    libxml2-dev \
    libxslt1-dev \
    libtool \
    unixodbc-dev \
    && rm -rf /var/lib/apt/lists/*

# 安装 libssl1.1 包
RUN dpkg -i /app/libssl1.1_1.1.1f-1ubuntu2.20_amd64.deb

# 给启动脚本执行权限
RUN chmod +x /app/start.sh

# 暴露所需的端口
EXPOSE 6000
EXPOSE 7000

# 设置默认环境变量
ENV PROTOCOL="optchain" \
    P2P="127.0.0.1:6000" \
    API="127.0.0.1:7000" \
    SHARD_ID="0" \
    PEERS="" \
    NODE_ID="0" \
    EXPER_NUMBER="0" \
    EXPER_ITER="0" \
    SHARD_NUM="5" \
    SHARD_SIZE="5" \
    BLOCK_SIZE="2048" \
    SYMBOL_SIZE="256" \
    PROP_SIZE="5" \
    AVAI_SIZE="5" \
    EX_REQ_NUM="3" \
    IN_REQ_NUM="1" \
    K="6" \
    DOMESTIC_RATE="0.7" \
    TX_DIFF="ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" \
    PROP_DIFF="ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" \
    AVAI_DIFF="ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" \
    IN_AVAI_DIFF="ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" \
    EX_DIFF="ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" \
    IN_DIFF="ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" 

# 设置容器启动时执行的命令
CMD ["/app/start.sh"]
