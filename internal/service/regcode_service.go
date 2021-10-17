package service

import (
	"context"
	pb "github.com/kr-juso/api/internal/grpc/juso/regcode"
	codes "google.golang.org/grpc/codes"
	status "google.golang.org/grpc/status"
)

type RegcodeService struct {
	pb.UnimplementedRegcodeServiceServer
}

func (service *RegcodeService) GetRegcodes(context.Context, *pb.GetRegcodesRequest) (*pb.GetRegcodesResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method GetRegcodes not implemented")
}
