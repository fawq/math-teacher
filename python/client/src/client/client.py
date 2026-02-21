import argparse
import logging
import secrets

import grpc

from client.teacher_pb2 import Numbers
from client.teacher_pb2_grpc import CalculatorStub

INT32_MAX = 2**31 - 1
INT32_MIN = -(2**31)


def main() -> int:
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
    args = argparser.parse_args()

    server_target = f"{args.host}:{args.port}"
    channel = grpc.insecure_channel(server_target)
    stub = CalculatorStub(channel)
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

    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        filename="client.log",
    )
    logger = logging.getLogger(__name__)

    logger.info("Connecting to server %s", server_target)

    try:
        if args.operation == "Mul":
            logger.info("Send multiplying %d and %d", number_1, number_2)
            response = stub.Mul(Numbers(Num1=number_1, Num2=number_2))
        elif args.operation == "Add":
            logger.info("Send adding %d and %d", number_1, number_2)
            response = stub.Add(Numbers(Num1=number_1, Num2=number_2))
        elif args.operation == "Sub":
            logger.info("Send subtracting %d and %d", number_1, number_2)
            response = stub.Sub(Numbers(Num1=number_1, Num2=number_2))
        else:
            msg = "Unknown operation: %s"
            logger.error(msg, args.operation)
            return 1
    except grpc.RpcError as e:
        logger.exception("gRPC error: %s - %s", e.code(), e.details())
        return 1

    logger.info("Received result: %s", response.result)
    return 0


if __name__ == "__main__":
    main()
