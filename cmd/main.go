package main

import (
	pb "github.com/kr-juso/api/internal/grpc/juso/regcode"
	"github.com/kr-juso/api/internal/service"
	"google.golang.org/grpc"
	"log"
	"net"
	"os"
)

func main() {
	portNumber := os.Getenv("PORT")
	lis, err := net.Listen("tcp", ":"+portNumber)
	if err != nil {
		log.Fatalf("failed to listen: %v", err)
	}

	grpcServer := grpc.NewServer()
	pb.RegisterRegcodeServiceServer(grpcServer, &service.RegcodeService{})

	log.Printf("start gPRC server on %s port", portNumber)
	if err := grpcServer.Serve(lis); err != nil {
		log.Fatalf("failed to listen: %v", err)
	}
}
