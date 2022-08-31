import csv
from email.header import Header
import os
from dataclasses import dataclass
from enum import Enum


class TransactionType(Enum):
    DEPOSIT = "deposit"
    WITHDRAWAL = "withdrawal"
    DISPUTE = "dispute"
    RESOLVE = "resolve"
    CHARGEBACK = "chargeback"


@dataclass
class Transaction:
    type: TransactionType
    client: int
    tx: int
    amount: float

    def as_csv_dict(self) -> dict:
        return {
            "type": self.type.value,
            "client": self.client,
            "tx": self.tx,
            "amount": float(self.amount),
        }

    def fieldnames() -> list:
        return ["type", "client", "tx", "amount"]


@dataclass
class Client:
    client: int
    available: float
    held: float
    total: float
    locked: bool

    def as_csv_dict(self) -> dict:
        return {
            "client": self.client,
            "available": float(self.available),
            "held": float(self.held),
            "total": float(self.total),
            "locked": "true" if self.locked else "false",
        }

    def fieldnames() -> list:
        return ["client", "available", "held", "total", "locked"]


def write_test_data(
    file_name: str,
    transactions: list[Transaction],
    final_accounts: list[Client],
    open_mode: str = "w",
):
    test_data_path = file_name + ".csv"
    with open(file_path(test_data_path), open_mode, newline="") as csvfile:
        writer = csv.DictWriter(
            csvfile, fieldnames=Transaction.fieldnames(), delimiter=","
        )
        writer.writeheader()
        for tx in transactions:
            writer.writerow(tx.as_csv_dict())

    test_data_expected_path = file_name + "_expected.csv"
    with open(file_path(test_data_expected_path), open_mode, newline="") as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=Client.fieldnames(), delimiter=",")
        writer.writeheader()
        for client in final_accounts:
            writer.writerow(client.as_csv_dict())


def file_path(rel_path: str) -> str:
    test_data_dir = os.path.join(os.path.dirname(__file__), "tests/test_data/")
    return os.path.join(test_data_dir, rel_path)


def deposit_withdrawl_test():
    file_name = "deposit_withdrawal"
    transactions = [
        Transaction(TransactionType.DEPOSIT, 1, 1, 1.0),
        Transaction(TransactionType.DEPOSIT, 2, 2, 2.0),
        Transaction(TransactionType.DEPOSIT, 1, 3, 2.0),
        Transaction(TransactionType.WITHDRAWAL, 1, 4, 1.5),
        Transaction(TransactionType.WITHDRAWAL, 2, 5, 3.0),
    ]
    final_accounts = [
        Client(1, 1.5, 0.0, 1.5, False),
        Client(2, 2.0, 0.0, 2.0, False),
    ]

    write_test_data(file_name, transactions, final_accounts)


def dispute_test():
    file_name = "dispute"
    transactions = [
        Transaction(TransactionType.DEPOSIT, 1, 1, 1.0),
        Transaction(TransactionType.DEPOSIT, 1, 2, 2.0),
        Transaction(TransactionType.DISPUTE, 1, 2, 0.0),
        Transaction(TransactionType.WITHDRAWAL, 1, 3, 2.0),
        Transaction(TransactionType.DEPOSIT, 1, 4, 10.0),
    ]
    final_accounts = [
        Client(1, 11.0, 2.0, 13.0, False),
    ]

    write_test_data(file_name, transactions, final_accounts)


def dispute_resolve_test():
    file_name = "dispute_resolve"
    transactions = [
        Transaction(TransactionType.DEPOSIT, 1, 1, 1.0),
        Transaction(TransactionType.DEPOSIT, 1, 2, 2.0),
        Transaction(TransactionType.DISPUTE, 1, 2, 0.0),
        Transaction(TransactionType.RESOLVE, 1, 2, 0.0),
        Transaction(TransactionType.WITHDRAWAL, 1, 3, 2.0),
    ]
    final_accounts = [
        Client(1, 1.0, 0.0, 1.0, False),
    ]

    write_test_data(file_name, transactions, final_accounts)


def dispute_charge_backtest():
    file_name = "dispute_chargeback"
    transactions = [
        Transaction(TransactionType.DEPOSIT, 1, 1, 1.0),
        Transaction(TransactionType.DEPOSIT, 1, 2, 2.0),
        Transaction(TransactionType.DISPUTE, 1, 2, 0.0),
        Transaction(TransactionType.CHARGEBACK, 1, 2, 0.0),
        Transaction(TransactionType.DEPOSIT, 1, 4, 10.0),
        Transaction(TransactionType.WITHDRAWAL, 1, 5, 1.0),
    ]
    final_accounts = [
        Client(1, 1.0, 0.0, 1.0, True),
    ]

    write_test_data(file_name, transactions, final_accounts)


def chungus():
    file_name = "chungus"
    num_rows = 1000

    test_data_path = file_name + ".csv"
    with open(file_path(test_data_path), "w", newline="") as csvfile:
        writer = csv.DictWriter(
            csvfile, fieldnames=Transaction.fieldnames(), delimiter=","
        )
        writer.writeheader()

        for i in range(1, num_rows + 1):
            tx = Transaction(TransactionType.DEPOSIT, 1, i, 1.0)
            writer.writerow(tx.as_csv_dict())

    test_data_expected_path = file_name + "_expected.csv"
    with open(file_path(test_data_expected_path), "w", newline="") as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=Client.fieldnames(), delimiter=",")
        writer.writeheader()

        client = Client(1, float(num_rows), 0.0, float(num_rows), False)
        writer.writerow(client.as_csv_dict())


if __name__ == "__main__":
    deposit_withdrawl_test()
    dispute_resolve_test()
    dispute_charge_backtest()
    dispute_test()
    chungus()
