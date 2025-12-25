PROTO_SRC := ../../protos
PROTO_OUT := src/agent_test_server/proto

.PHONY: proto clean

proto:
	python -m grpc_tools.protoc \
		-I$(PROTO_SRC) \
		--python_out=$(PROTO_OUT) \
		--pyi_out=$(PROTO_OUT) \
		$(PROTO_SRC)/deploything/v1/remote_command.proto

clean:
	rm -f $(PROTO_OUT)/deploything/v1/remote_command_pb2.py
	rm -f $(PROTO_OUT)/deploything/v1/remote_command_pb2.pyi
