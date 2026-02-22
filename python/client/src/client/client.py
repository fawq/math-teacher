import argparse
import logging
import secrets

import grpc

from client.teacher_pb2 import Numbers, Result
from client.teacher_pb2_grpc import CalculatorStub

INT32_MAX = 2**31 - 1
INT32_MIN = -(2**31)


def argument_parser() -> argparse.Namespace:
    argparser = argparse.ArgumentParser(description="Math Teacher gRPC Client")
    argparser.add_argument(
        "--host", type=str, default="[::1]", help="server host (default: [::1])"
    )
    argparser.add_argument(
        "--port", type=int, default=10000, help="server port (default: 10000)"
    )
    argparser.add_argument(
        "--operation",
        type=str,
        default="Mul",
        help="operation to perform (default: Mul)",
    )
    argparser.add_argument(
        "--numbers",
        type=int,
        nargs=2,
        help="two numbers to perform the operation on (default: random)",
    )
    argparser.add_argument(
        "--iterations",
        type=int,
        default=1,
        help="number of iterations to perform (default: 1)",
    )
    return argparser.parse_args()


def get_logger() -> logging.Logger:
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        handlers=[logging.StreamHandler(), logging.FileHandler("client.log")],
    )

    return logging.getLogger(__name__)


def wait_for_server(channel: grpc.Channel, timeout: float = 20.0) -> bool:
    try:
        grpc.channel_ready_future(channel).result(timeout=timeout)
    except grpc.FutureTimeoutError:
        return False
    else:
        return True


def get_response(
    stub: CalculatorStub, operation: str, number_1: int, number_2: int
) -> Result:
    if operation == "Mul":
        return stub.Mul(Numbers(Num1=number_1, Num2=number_2))
    if operation == "Add":
        return stub.Add(Numbers(Num1=number_1, Num2=number_2))
    if operation == "Sub":
        return stub.Sub(Numbers(Num1=number_1, Num2=number_2))
    msg = f"Unknown operation: {operation}"
    raise ValueError(msg)


def main() -> int:
    args = argument_parser()
    logger = get_logger()

    server_target = f"{args.host}:{args.port}"
    channel = grpc.insecure_channel(server_target)
    if not wait_for_server(channel):
        logger.error("Failed to connect to server %s", server_target)
        return 1
    stub = CalculatorStub(channel)
    logger.info("Connecting to server %s", server_target)

    number_1 = (
        args.numbers[0]
        if args.numbers
        else secrets.randbelow(INT32_MAX - INT32_MIN) + INT32_MIN
    )
    number_2 = (
        args.numbers[1]
        if args.numbers
        else secrets.randbelow(INT32_MAX - INT32_MIN) + INT32_MIN
    )

    try:
        for _ in range(args.iterations):
            response = get_response(stub, args.operation, number_1, number_2)
            logger.info("Received result: %s", response.result)
    except grpc.RpcError as e:
        logger.exception("gRPC error: %s - %s", e.code(), e.details())
        return 1
    except Exception:
        logger.exception("Unexpected error occurred")
        return 1
    return 0


if __name__ == "__main__":
    main()
