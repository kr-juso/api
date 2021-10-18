// Code generated by protoc-gen-go-grpc. DO NOT EDIT.

package regcode

import (
	context "context"
	grpc "google.golang.org/grpc"
	codes "google.golang.org/grpc/codes"
	status "google.golang.org/grpc/status"
)

// This is a compile-time assertion to ensure that this generated file
// is compatible with the grpc package it is being compiled against.
// Requires gRPC-Go v1.32.0 or later.
const _ = grpc.SupportPackageIsVersion7

// RegcodeServiceClient is the client API for RegcodeService service.
//
// For semantics around ctx use and closing/ending streaming RPCs, please refer to https://pkg.go.dev/google.golang.org/grpc/?tab=doc#ClientConn.NewStream.
type RegcodeServiceClient interface {
	GetRegcodes(ctx context.Context, in *GetRegcodesRequest, opts ...grpc.CallOption) (*GetRegcodesResponse, error)
}

type regcodeServiceClient struct {
	cc grpc.ClientConnInterface
}

func NewRegcodeServiceClient(cc grpc.ClientConnInterface) RegcodeServiceClient {
	return &regcodeServiceClient{cc}
}

func (c *regcodeServiceClient) GetRegcodes(ctx context.Context, in *GetRegcodesRequest, opts ...grpc.CallOption) (*GetRegcodesResponse, error) {
	out := new(GetRegcodesResponse)
	err := c.cc.Invoke(ctx, "/juso.regcode.RegcodeService/GetRegcodes", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

// RegcodeServiceServer is the server API for RegcodeService service.
// All implementations must embed UnimplementedRegcodeServiceServer
// for forward compatibility
type RegcodeServiceServer interface {
	GetRegcodes(context.Context, *GetRegcodesRequest) (*GetRegcodesResponse, error)
	mustEmbedUnimplementedRegcodeServiceServer()
}

// UnimplementedRegcodeServiceServer must be embedded to have forward compatible implementations.
type UnimplementedRegcodeServiceServer struct {
}

func (UnimplementedRegcodeServiceServer) GetRegcodes(context.Context, *GetRegcodesRequest) (*GetRegcodesResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method GetRegcodes not implemented")
}
func (UnimplementedRegcodeServiceServer) mustEmbedUnimplementedRegcodeServiceServer() {}

// UnsafeRegcodeServiceServer may be embedded to opt out of forward compatibility for this service.
// Use of this interface is not recommended, as added methods to RegcodeServiceServer will
// result in compilation errors.
type UnsafeRegcodeServiceServer interface {
	mustEmbedUnimplementedRegcodeServiceServer()
}

func RegisterRegcodeServiceServer(s grpc.ServiceRegistrar, srv RegcodeServiceServer) {
	s.RegisterService(&RegcodeService_ServiceDesc, srv)
}

func _RegcodeService_GetRegcodes_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(GetRegcodesRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(RegcodeServiceServer).GetRegcodes(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/juso.regcode.RegcodeService/GetRegcodes",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(RegcodeServiceServer).GetRegcodes(ctx, req.(*GetRegcodesRequest))
	}
	return interceptor(ctx, in, info, handler)
}

// RegcodeService_ServiceDesc is the grpc.ServiceDesc for RegcodeService service.
// It's only intended for direct use with grpc.RegisterService,
// and not to be introspected or modified (even as a copy)
var RegcodeService_ServiceDesc = grpc.ServiceDesc{
	ServiceName: "juso.regcode.RegcodeService",
	HandlerType: (*RegcodeServiceServer)(nil),
	Methods: []grpc.MethodDesc{
		{
			MethodName: "GetRegcodes",
			Handler:    _RegcodeService_GetRegcodes_Handler,
		},
	},
	Streams:  []grpc.StreamDesc{},
	Metadata: "api/protobuf/juso/juso.proto",
}